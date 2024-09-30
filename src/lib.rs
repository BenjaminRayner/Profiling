#![warn(clippy::all)]
pub mod checksum;
pub mod idea;
pub mod package;
pub mod student;

use idea::Idea;
use package::Package;

pub enum PkgEvent {
    // Packages that students can take to work on their ideas
    DownloadComplete(Package)
}

pub enum IdeaEvent {
    // Newly generated idea for students to work on
    NewIdea(Idea),
    // Termination event for student threads
    OutOfIdeas
}
