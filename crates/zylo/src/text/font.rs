use owned_ttf_parser::{AsFaceRef, Face, OwnedFace};
use rustc_hash::FxHashMap;

/// Unique ID of a font in a `FontStore`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FontId(fontdb::ID);

/// Stores all fonts registered with a `Context`.
pub struct FontStore {
    db: fontdb::Database,
    fonts: FxHashMap<FontId, OwnedFace>,
}

impl FontStore {
    pub fn new() -> Self {
        Self {
            db: fontdb::Database::new(),
            fonts: FxHashMap::default(),
        }
    }

    pub fn load_system_fonts(&mut self) {
        self.db.load_system_fonts();
    }

    pub fn load_font_from_data(&mut self, data: impl Into<Vec<u8>>) {
        self.db.load_font_data(data.into());
    }

    /// Queries for a font based on parameters.
    pub fn query(&self, query: &fontdb::Query) -> Option<FontId> {
        self.db.query(query).map(FontId)
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
