use fontdb::{Family, Stretch, Style, Weight};
use owned_ttf_parser::{AsFaceRef, Face, OwnedFace};
use rustc_hash::FxHashMap;

use crate::FontFamily;

/// Unique ID of a font in a `FontStore`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FontId(fontdb::ID);

/// A query for a font.
#[derive(Debug)]
pub struct Query<'a> {
    pub font_family: FontFamily<'a>,
    pub weight: Weight,
    pub italic: bool,
}

/// Stores all fonts registered with a `Context`.
pub struct FontStore {
    db: fontdb::Database,
    fonts: FxHashMap<FontId, OwnedFace>,

    keyed_fonts: FxHashMap<String, String>,
}

impl FontStore {
    pub fn new() -> Self {
        Self {
            db: fontdb::Database::new(),
            fonts: FxHashMap::default(),
            keyed_fonts: FxHashMap::default(),
        }
    }

    pub fn load_system_fonts(&mut self) {
        self.db.load_system_fonts();
    }

    pub fn load_font_from_data(&mut self, data: impl Into<Vec<u8>>) {
        self.db.load_font_data(data.into());
    }

    pub fn add_keyed_font(&mut self, key: impl Into<String>, font_family: impl Into<String>) {
        self.keyed_fonts.insert(key.into(), font_family.into());
    }

    /// Queries for a font based on parameters.
    pub fn query(&self, query: &Query) -> Option<FontId> {
        self.with_converted_query(query, |query| self.db.query(&query).map(FontId))
    }

    fn with_converted_query<R>(
        &self,
        query: &Query,
        callback: impl FnOnce(fontdb::Query) -> R,
    ) -> R {
        let font_family = if query.font_family.is_keyed() {
            self.keyed_fonts[query.font_family.name_or_key()].as_str()
        } else {
            query.font_family.name_or_key()
        };
        let families = [Family::Name(font_family)];
        let query = fontdb::Query {
            families: &families,
            weight: query.weight,
            stretch: Stretch::default(),
            style: if query.italic {
                Style::Italic
            } else {
                Style::Normal
            },
        };

        callback(query)
    }

    /// Gets a ttf_parser::Face from a font ID.
    pub fn get(&mut self, id: FontId) -> &Face {
        self.fonts
            .entry(id)
            .or_insert_with(|| {
                self.db
                    .with_face_data(id.0, |data, index| {
                        OwnedFace::from_vec(data.to_vec(), index).expect("malformed font")
                    })
                    .expect("invalid font ID")
            })
            .as_face_ref()
    }
}
