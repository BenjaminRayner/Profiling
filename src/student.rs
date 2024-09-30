use super::{checksum::Checksum, idea::Idea, package::Package, PkgEvent, IdeaEvent};
use crossbeam::channel::{Receiver, Sender};

pub struct Student {
    idea: Option<Idea>,
    pkgs: Vec<Package>,
    pkg_event_sender: Sender<PkgEvent>,
    pkg_event_recv: Receiver<PkgEvent>,
    idea_event_recv: Receiver<IdeaEvent>
}

impl Student {
    pub fn new(pkg_event_sender: Sender<PkgEvent>, pkg_event_recv: Receiver<PkgEvent>, idea_event_recv: Receiver<IdeaEvent>) -> Self {
        Self {
            pkg_event_sender,
            pkg_event_recv,
            idea_event_recv,
            idea: None,
            pkgs: vec![]
        }
    }

    fn build_idea(&mut self, idea_checksum: &mut Checksum, pkg_checksum: &mut Checksum) {
        if let Some(ref idea) = self.idea {
            // Can only build ideas if we have acquired sufficient packages
            let pkgs_required = idea.num_pkg_required;
            if pkgs_required <= self.pkgs.len() {
                // Update idea and package checksums
                // All of the packages used in the update are deleted, along with the idea
                idea_checksum.update(Checksum::with_sha256(&idea.name));
                let pkgs_used = self.pkgs.drain(0..pkgs_required).collect::<Vec<_>>();
                for pkg in pkgs_used.iter() {
                    pkg_checksum.update(Checksum::with_sha256(&pkg.name));
                }

                self.idea = None; // Get ready to work on new idea
            }
        }
    }

    pub fn run(&mut self) -> (Checksum, Checksum) {

        let mut pkg_checksum = Checksum::default();
        let mut idea_checksum = Checksum::default();

        loop 
        {
            // Check for new packages
            let pkg_event = self.pkg_event_recv.try_recv();
            if pkg_event.is_ok() {
                let pkg_event = pkg_event.unwrap();
                match pkg_event {
                    PkgEvent::DownloadComplete(pkg) => {
                        self.pkgs.push(pkg);
                        self.build_idea(&mut idea_checksum, &mut pkg_checksum);
                    }
                }
            }

            // If we don't have an idea, check for new ideas
            if self.idea.is_none() {
                let idea_event = self.idea_event_recv.try_recv();
                if idea_event.is_ok() {
                    let idea_event = idea_event.unwrap();
                    match idea_event {
                        IdeaEvent::NewIdea(idea) => {
                            self.idea = Some(idea);
                            self.build_idea(&mut idea_checksum, &mut pkg_checksum);
                        }
                        IdeaEvent::OutOfIdeas => {
                            for pkg in self.pkgs.drain(..) {
                                self.pkg_event_sender.send(PkgEvent::DownloadComplete(pkg)).unwrap();
                            }
                            return (idea_checksum, pkg_checksum);
                        }
                    }
                }
            }
        }
    }
}
