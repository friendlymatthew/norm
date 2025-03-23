use crate::font::grammar::{
    Glyph,
    TrueTypeFontFile,
};

#[derive(Debug)]
pub struct TrueTypeFontShaper<'a> {
    file: &'a TrueTypeFontFile<'a>,
}

impl<'a> TrueTypeFontShaper<'a> {
    pub const fn from(file: &'a TrueTypeFontFile<'a>) -> Self {
        Self { file }
    }

    pub fn shape(&self, phrase: &str) -> Vec<&Glyph> {
        let cmap_4 = self
            .file
            .cmap_table
            .format_4()
            .expect("How do you handle other formats?");

        let words = phrase.split_whitespace();

        words
            .flat_map(|word| {
                word.chars()
                    .map(|c| {
                        let j = cmap_4.find_glyph_index(c);

                        &self.file.glyph_table.glyphs[j]
                    })
                    .collect::<Vec<&Glyph>>()
            })
            .collect::<Vec<&Glyph>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::TrueTypeFontParser;
    use std::fs;

    #[test]
    fn test_parse_lato() -> anyhow::Result<()> {
        let ttf_file = fs::read("./src/font/Lato-Regular.ttf")?;
        let ttf = TrueTypeFontParser::new(&ttf_file).parse()?;

        let shaper = TrueTypeFontShaper::from(&ttf);

        let glyphs = shaper.shape("hello world!");

        dbg!(glyphs);

        Ok(())
    }
}
