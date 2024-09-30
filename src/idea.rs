use super::checksum::Checksum;
use super::IdeaEvent;
use crossbeam::channel::Sender;

pub struct Idea {
    pub name: String,
    pub num_pkg_required: usize,
}

pub struct IdeaGenerator {
    ideas: Vec<String>,
    idea_start_idx: usize,
    num_ideas: usize,
    num_students: usize,
    num_pkgs: usize,
    event_sender: Sender<IdeaEvent>
}

impl IdeaGenerator {
    pub fn new(
        ideas: Vec<String>,
        idea_start_idx: usize,
        num_ideas: usize,
        num_students: usize,
        num_pkgs: usize,
        event_sender: Sender<IdeaEvent>
    ) -> Self {
        Self {
            ideas,
            idea_start_idx,
            num_ideas,
            num_students,
            num_pkgs,
            event_sender
        }
    }

    pub fn run(&self)  -> Checksum{
        let mut idea_checksum = Checksum::default();

        let pkg_per_idea = self.num_pkgs / self.num_ideas;
        let extra_pkgs = self.num_pkgs % self.num_ideas;

        // Generate a set of new ideas and place them into the event-queue
        // Update the idea checksum with all generated idea names
        for i in 0..self.num_ideas {
            let name = self.ideas[(self.idea_start_idx + i) % self.ideas.len()].to_owned();
            let extra = (i < extra_pkgs) as usize;
            let num_pkg_required = pkg_per_idea + extra;
            let idea = Idea {
                name,
                num_pkg_required,
            };

            idea_checksum.update(Checksum::with_sha256(&idea.name));

            self.event_sender.send(IdeaEvent::NewIdea(idea)).unwrap();
        }

        // Push student termination events into the event queue
        for _ in 0..self.num_students {
            self.event_sender.send(IdeaEvent::OutOfIdeas).unwrap();
        }

        idea_checksum
    }
}
