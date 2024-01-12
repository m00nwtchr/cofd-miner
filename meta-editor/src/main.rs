use std::{
	collections::BTreeMap,
	fs::{self, File},
	ops::Range,
	path::{Path, PathBuf},
};

use eframe::egui::text::{LayoutSection, TextWrapping};
use eframe::{
	egui::{self, FontSelection, TextEdit, TextFormat},
	epaint::{self, Color32, FontId},
};
use serde::Serialize;
use serde_json::ser::PrettyFormatter;

use cofd_meta::{Op, PageKind, SectionMeta, SectionRange, SourceMeta};
use cofd_miner::{hash, process_section};
use cofd_miner::source::Section;
use cofd_schema::prelude::BookInfo;

fn main() -> eframe::Result<()> {
	let native_options = eframe::NativeOptions::default();
	eframe::run_native(
		"My egui App",
		native_options,
		Box::new(|cc| Box::new(MetaEditorApp::new(cc))),
	)
}

struct MetaEditorApp {
	meta: SourceMeta,
	meta_path: PathBuf,
	pages: BTreeMap<usize, Vec<String>>,
	path: PathBuf,

	selected_section: Option<usize>,
	section: Option<Section>,
	selected_op: Option<usize>,
	show_full_text: bool,
	last_range: Option<Range<usize>>,
	pages_start: String,
	pages_end: String,
}

impl MetaEditorApp {
	fn new(cc: &eframe::CreationContext<'_>) -> Self {
		let args: Vec<_> = std::env::args().collect();
		let path = PathBuf::from(args.get(1).unwrap());

		let hash = hash::hash(&path).unwrap();

		let (meta, meta_path) = fs::read_dir("meta")
			.unwrap()
			.filter_map(|entry| entry.ok().map(|e| e.path()))
			.filter(|path| path.extension().map(|ext| ext.eq("json")).unwrap_or(false))
			.map(|path| -> anyhow::Result<(SourceMeta, PathBuf)> {
				Ok((serde_json::de::from_reader(File::open(&path)?)?, path))
			})
			.filter_map(|r| r.ok())
			.find(|(meta, path)| meta.info.hash.eq(&hash))
			.unwrap_or_else(|| {
				(
					SourceMeta {
						info: BookInfo {
							hash,
							..Default::default()
						},
						sections: Vec::new(),
					},
					Path::new("meta")
						.join(path.file_name().unwrap())
						.with_extension("json"),
				)
			});

		let pages = cofd_miner::extract_pages(&path).unwrap();

		Self {
			meta,
			meta_path,
			pages,
			path,
			selected_section: None,
			section: None,
			selected_op: None,
			show_full_text: true,
			last_range: None,
			pages_end: String::new(),
			pages_start: String::new(),
		}
	}

	fn highlight(
		show_full_text: bool,
		ui: &egui::Ui,
		text: &str,
		font_id: FontId,
		wrap_width: f32,
		section: &SectionMeta,
	) -> epaint::text::LayoutJob {
		let mut layout_job = epaint::text::LayoutJob {
			sections: vec![LayoutSection {
				leading_space: 0.0,
				byte_range: 0..text.len(),
				format: TextFormat {
					background: Color32::TRANSPARENT,
					color: Color32::GRAY,
					..Default::default()
				},
			}],
			text: text.to_string(),
			wrap: TextWrapping {
				max_width: wrap_width,
				..Default::default()
			},
			break_on_newline: true,
			..Default::default()
		};

		if show_full_text {
			if let Some(range) = &section.range {
				layout_job.sections.clear();
				layout_job.text = String::new();

				for (i, line) in text.split('\n').enumerate() {
					let format = match range {
						SectionRange::Range(range) => {
							if range.contains(&i) {
								TextFormat {
									background: Color32::GRAY,
									color: Color32::BLACK,
									..Default::default()
								}
							} else {
								TextFormat::default()
							}
						}
						SectionRange::Regex(regex) => TextFormat::default(),
					};

					layout_job.append(line, 0.0, format.clone());
					layout_job.append("\n", 0.0, format);
				}
			}
		}

		layout_job
	}
}

impl eframe::App for MetaEditorApp {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		egui::SidePanel::left("sidebar")
			.resizable(false)
			.show(ctx, |ui| {
				ui.label("Sections:");
				for (i, section) in self.meta.sections.iter().enumerate() {
					if ui
						.selectable_value(&mut self.selected_section, Some(i), &section.name)
						.clicked()
					{
						let section_def = self.meta.sections.get(self.selected_section.unwrap()).unwrap();
						self.section = Some(process_section(&self.pages, section_def, self.show_full_text).unwrap());

						if let Some(selection) = self
							.selected_section
							.and_then(|selected_section| self.meta.sections.get(selected_section))
						{
							self.pages_start = selection.pages.start().to_string();
							self.pages_end = selection.pages.end().to_string();
						}
					}
				}
				ui.separator();

				if let Some(selected_section) = self.selected_section {
					if let Some(section) = self.meta.sections.get_mut(selected_section) {
						ui.text_edit_singleline(&mut section.name);

						ui.horizontal_top(|ui| {
							if ui
								.add(
									TextEdit::singleline(&mut self.pages_start)
										.id_source("pages_start"),
								)
								.changed()
							{
								section.pages =
									(self.pages_start.parse().unwrap())..=*section.pages.end();
							}
							if ui
								.add(
									TextEdit::singleline(&mut self.pages_end)
										.id_source("pages_end"),
								)
								.changed()
							{
								section.pages =
									*section.pages.start()..=(self.pages_end.parse().unwrap())
							}
						});
						ui.separator();

						ui.label("Operations:");
						for (i, op) in section.ops.iter().enumerate() {
							ui.selectable_value(
								&mut self.selected_op,
								Some(i),
								match op {
									Op::Insert { .. } => "Insert",
									Op::Delete { .. } => "Delete",
									Op::Move { .. } => "Move",
									Op::RegexReplace { .. } => "RegexReplace",
									Op::Replace { .. } => "Replace",
								},
							);
						}
						ui.separator();

						if let Some(selection) = self
							.selected_op
							.and_then(|selected_op| section.ops.get_mut(selected_op))
						{
							match selection {
								Op::Insert { pos: _, char: _ } => {}
								Op::Delete { range: _ } => {}
								Op::Move { range: _, pos: _ } => {}
								Op::RegexReplace { regex, replace: _ } => {
									ui.text_edit_singleline(regex);
								}
								Op::Replace { range: _, replace } => {
									ui.text_edit_singleline(replace);
								}
							}
						}
					}
				}

				if ui.button("Add section").clicked() {
					self.meta.sections.push(SectionMeta {
						name: String::from("Unnamed"),
						pages: 1..=2,
						range: None,
						kind: PageKind::Merit(None),
						ops: Vec::new(),
					})
				}

				if ui.button("Save").clicked() {
					let mut ser = serde_json::Serializer::with_formatter(
						File::create(&self.meta_path).unwrap(),
						PrettyFormatter::with_indent(b"\t"),
					);
					self.meta.serialize(&mut ser);
				}

				ui.checkbox(&mut self.show_full_text, "Show full text");
			});

		egui::CentralPanel::default().show(ctx, |ui| {
			egui::ScrollArea::vertical()
				// .id_source("source")
				.show(ui, |ui| {
					if let (Some(selected_section), Some(section)) = (self.selected_section, &self.section) {
						// let mut text: &str = self.pages.get(&2).unwrap().as_str();
						let section_def = self.meta.sections.get_mut(selected_section).unwrap();
						// let sec = section_def.clone();

						let font_id = FontSelection::Default.resolve(ui.style());
						let mut layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
							let mut layout_job = MetaEditorApp::highlight(
								self.show_full_text,
								ui,
								text,
								font_id.clone(),
								wrap_width,
								section_def,
							);
							layout_job.wrap.max_width = wrap_width;
							ui.fonts(|f| f.layout_job(layout_job))
						};

						let mut text = section.extract.as_str();
						use egui::TextBuffer as _;

						let output = egui::TextEdit::multiline(&mut text)
							.layouter(&mut layouter)
							.desired_width(f32::INFINITY)
							.show(ui);

						if let Some(cursor_range) = output.cursor_range {
							if !cursor_range.is_empty() {
								let [start, end] = cursor_range.sorted_cursors();
								let start = text.byte_index_from_char_index(start.ccursor.index);
								let end = text.byte_index_from_char_index(end.ccursor.index);

								self.last_range = Some(start..end);
							}
						}

						output.response.context_menu(|ui| {
							if ui.button("Set range").clicked() {
								if let Some(range) = &self.last_range {
									let start = text[0..range.start]
										.chars()
										.filter(|c| c.eq(&'\n'))
										.count();

									let text = &text[range.clone()];
									let end = text.chars().filter(|c| c.eq(&'\n')).count()
										+ if text.ends_with('\n') { 0 } else { 1 };

									println!("{start}");
									section_def.range =
										Some(SectionRange::Range(start..(start + end)));
								}

								ui.close_menu();
							}

							if ui.button("Delete").clicked() {
								if let Some(range) = &self.last_range {
									let range = range.start..=(range.end - 1);

									section_def.ops.push(Op::Delete { range })
								}

								ui.close_menu();
							}

							if ui.button("Replace").clicked() {
								if let Some(range) = &self.last_range {
									let range = range.start..=(range.end - 1);

									section_def.ops.push(Op::Replace {
										range,
										replace: String::new(),
									})
								}

								ui.close_menu();
							}
						});
					}
				});
		});
	}
}
