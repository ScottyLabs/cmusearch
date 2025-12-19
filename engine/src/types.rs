use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Course data structure matching courses.json format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Course {
    #[serde(rename = "courseID")]
    pub course_id: String,
    pub name: String,
    pub desc: String,
    pub department: String,
    pub units: String,
    #[serde(default)]
    pub prereqs: Vec<String>,
    #[serde(rename = "prereqString", default)]
    pub prereq_string: String,
    #[serde(default)]
    pub coreqs: Vec<String>,
    #[serde(default)]
    pub crosslisted: Vec<String>,
}

/// Floor info for room documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Floor {
    #[serde(rename = "buildingCode", default)]
    pub building_code: String,
    #[serde(default)]
    pub level: String,
}

/// Room/Building document matching roomDocuments.json format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomDocument {
    pub id: String,
    #[serde(rename = "nameWithSpace", default)]
    pub name: String,
    #[serde(rename = "fullNameWithSpace", default)]
    pub full_name: String,
    #[serde(rename = "type", default)]
    pub room_type: String,
    #[serde(default)]
    pub alias: String,
    #[serde(rename = "numTerms", default)]
    pub num_terms: u16,
    #[serde(default)]
    pub floor: Option<Floor>,
}

/// Unified document enum for search results
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "doc_type")]
pub enum Document {
    Course(Course),
    Room(RoomDocument),
}

/// Search result with document info and relevance score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub document: Document,
    pub score: f32,
}

/// Inverted index: term -> list of (document_id, term_frequency)
pub type InvertedIndex = HashMap<String, Vec<(String, u16)>>;

/// Document store: document_id -> Document
pub type DocumentStore = HashMap<String, Document>;

/// Course store: document_id -> Course
pub type CourseStore = HashMap<String, Course>;

/// Room store: document_id -> RoomDocument
pub type RoomStore = HashMap<String, RoomDocument>;
