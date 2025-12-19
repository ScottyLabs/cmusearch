use crate::parse::text_to_trigrams;
use crate::types::{
    Course, CourseStore, Document, DocumentStore, InvertedIndex, RoomDocument, RoomStore,
};
use std::collections::HashMap;

/// Get searchable text from a Course
fn course_searchable_text(course: &Course) -> String {
    format!(
        "{} {} {} {}",
        course.course_id, course.name, course.desc, course.department
    )
}

/// Get searchable text from a RoomDocument
fn room_searchable_text(room: &RoomDocument) -> String {
    format!(
        "{} {} {} {}",
        room.name, room.full_name, room.alias, room.room_type
    )
}

/// Get searchable text from a Document
pub fn document_searchable_text(doc: &Document) -> String {
    match doc {
        Document::Course(c) => course_searchable_text(c),
        Document::Room(r) => room_searchable_text(r),
    }
}

/// Build inverted index from documents
/// Maps each trigram to list of (doc_id, term_frequency) pairs
pub fn build_index(docs: &DocumentStore) -> InvertedIndex {
    let mut index: InvertedIndex = HashMap::new();

    for (doc_id, doc) in docs {
        let searchable_text = document_searchable_text(doc);

        // Generate trigrams and count frequencies
        let trigrams = text_to_trigrams(&searchable_text);
        let mut term_freqs: HashMap<String, u16> = HashMap::new();

        for trigram in trigrams {
            *term_freqs.entry(trigram).or_insert(0) += 1;
        }

        // Add to inverted index
        for (term, freq) in term_freqs {
            index
                .entry(term)
                .or_insert_with(Vec::new)
                .push((doc_id.clone(), freq));
        }
    }

    index
}

/// Calculate the number of trigrams in a document (for doc length)
pub fn get_doc_length(doc: &Document) -> u16 {
    let searchable_text = document_searchable_text(doc);
    text_to_trigrams(&searchable_text).len() as u16
}

/// Convert CourseStore to DocumentStore
pub fn courses_to_docs(courses: CourseStore) -> DocumentStore {
    courses
        .into_iter()
        .map(|(id, c)| (format!("course:{}", id), Document::Course(c)))
        .collect()
}

/// Convert RoomStore to DocumentStore
pub fn rooms_to_docs(rooms: RoomStore) -> DocumentStore {
    rooms
        .into_iter()
        .map(|(id, r)| (format!("room:{}", id), Document::Room(r)))
        .collect()
}

/// Merge two document stores
pub fn merge_docs(mut a: DocumentStore, b: DocumentStore) -> DocumentStore {
    a.extend(b);
    a
}
