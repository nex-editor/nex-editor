use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct Document {
    sections: Vec<Section>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Section {
    page_height: f32,
    content: Vec<Block>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum Block {
    Paragraph(Paragraph),
    Control(ControlChar),
    Table(Table),
}

#[derive(Debug, Serialize, Deserialize)]
struct Paragraph {
    runs: Vec<Run>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Run {
    text: String,
    style: Style,
}

#[derive(Debug, Serialize, Deserialize)]
struct Style {
    bold: bool,
    italic: bool,
    font_size: f32,
}

#[derive(Debug, Serialize, Deserialize)]
enum ControlChar {
    PageBreak,
}

#[derive(Debug, Serialize, Deserialize)]
struct Table {
    rows: Vec<TableRow>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TableRow {
    cells: Vec<TableCell>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TableCell {
    content: Vec<Block>,
}

struct Cursor {
    section_index: usize,
    block_index: usize,
    run_index: usize,
    char_offset: usize,
}

impl Document {
    pub fn new() -> Self {
        Self { sections: vec![] }
    }

    pub fn insert_section(&mut self, section: Section, index: usize) {
        self.sections.insert(index, section);
    }

    pub fn remove_section(&mut self, index: usize) {
        self.sections.remove(index);
    }

    pub fn get_section(&self, index: usize) -> Option<&Section> {
        self.sections.get(index)
    }

    pub fn get_section_mut(&mut self, index: usize) -> Option<&mut Section> {
        self.sections.get_mut(index)
    }
}

impl Cursor {
    pub fn move_to(&mut self, section: usize, block: usize, run: usize, char_offset: usize) {
        self.section_index = section;
        self.block_index = block;
        self.run_index = run;
        self.char_offset = char_offset;
    }

    pub fn move_next_char(&mut self, doc: &Document) {
        if let Some(run) = doc.sections.get(self.section_index)
            .and_then(|s| s.content.get(self.block_index))
            .and_then(|b| match b {
                Block::Paragraph(p) => p.runs.get(self.run_index),
                _ => None,
            }) 
        {
            if self.char_offset < run.text.len() {
                self.char_offset += 1;
            } else {
                // 自动跳到下一个 Run 或 Block
            }
        }
    }

    pub fn move_prev_char(&mut self) {
        if self.char_offset > 0 {
            self.char_offset -= 1;
        }
    }
}

impl Document {
    pub fn insert_text(&mut self, cursor: &mut Cursor, text: &str, style: Style) {
        if let Some(run) = self.sections.get_mut(cursor.section_index)
            .and_then(|s| s.content.get_mut(cursor.block_index))
            .and_then(|b| match b {
                Block::Paragraph(p) => p.runs.get_mut(cursor.run_index),
                _ => None,
            }) 
        {
            run.text.insert_str(cursor.char_offset, text);
            cursor.char_offset += text.len();
        }
    }

    pub fn delete_char(&mut self, cursor: &mut Cursor) {
        if let Some(run) = self.sections.get_mut(cursor.section_index)
            .and_then(|s| s.content.get_mut(cursor.block_index))
            .and_then(|b| match b {
                Block::Paragraph(p) => p.runs.get_mut(cursor.run_index),
                _ => None,
            }) 
        {
            if cursor.char_offset > 0 {
                run.text.remove(cursor.char_offset - 1);
                cursor.char_offset -= 1;
            }
        }
    }
}


impl Document {
    pub fn toggle_bold(&mut self, cursor: &Cursor) {
        if let Some(run) = self.sections.get_mut(cursor.section_index)
            .and_then(|s| s.content.get_mut(cursor.block_index))
            .and_then(|b| match b {
                Block::Paragraph(p) => p.runs.get_mut(cursor.run_index),
                _ => None,
            }) 
        {
            run.style.bold = !run.style.bold;
        }
    }

    pub fn set_font_size(&mut self, cursor: &Cursor, size: f32) {
        if let Some(run) = self.sections.get_mut(cursor.section_index)
            .and_then(|s| s.content.get_mut(cursor.block_index))
            .and_then(|b| match b {
                Block::Paragraph(p) => p.runs.get_mut(cursor.run_index),
                _ => None,
            }) 
        {
            run.style.font_size = size;
        }
    }
}


impl Document {
    pub fn insert_page_break(&mut self, cursor: &Cursor) {
        if let Some(section) = self.sections.get_mut(cursor.section_index) {
            section.content.insert(cursor.block_index + 1, Block::Control(ControlChar::PageBreak));
        }
    }
}
