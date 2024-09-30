use super::checksum::Checksum;
use super::PkgEvent;
use crossbeam::channel::Sender;

pub struct Package {
    pub name: String,
}

pub struct PackageDownloader {
    pkgs: Vec<String>,
    pkg_start_idx: usize,
    num_pkgs: usize,
    event_sender: Sender<PkgEvent>
}

impl PackageDownloader {
    pub fn new(pkgs: Vec<String>, pkg_start_idx: usize, num_pkgs: usize, event_sender: Sender<PkgEvent>) -> Self {
        Self {
            pkgs,
            pkg_start_idx,
            num_pkgs,
            event_sender
        }
    }

    pub fn run(&self) -> Checksum {
        let mut pkg_checksum = Checksum::default();

        // Generate a set of packages and place them into the event queue
        // Update the package checksum with each package name
        for i in 0..self.num_pkgs {
            let name = self.pkgs[(self.pkg_start_idx + i) % self.pkgs.len()].to_owned();

            pkg_checksum.update(Checksum::with_sha256(&name));
            
            self.event_sender.send(PkgEvent::DownloadComplete(Package { name })).unwrap();
        }

        pkg_checksum
    }
}
