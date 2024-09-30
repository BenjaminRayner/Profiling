#![warn(clippy::all)]
use lab4::{
    checksum::Checksum, idea::IdeaGenerator, package::PackageDownloader, student::Student, PkgEvent, IdeaEvent
};
use crossbeam::channel::{bounded, Receiver, Sender};
use std::fs;
use std::env;
use std::error::Error;
use std::thread::spawn;

struct Args {
    pub num_ideas: usize,
    pub num_idea_gen: usize,
    pub num_pkgs: usize,
    pub num_pkg_gen: usize,
    pub num_students: usize,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args().collect();
    let num_ideas = args.get(1).map_or(Ok(80), |a| a.parse())?;
    let num_idea_gen = args.get(2).map_or(Ok(2), |a| a.parse())?;
    let num_pkgs = args.get(3).map_or(Ok(4000), |a| a.parse())?;
    let num_pkg_gen = args.get(4).map_or(Ok(6), |a| a.parse())?;
    let num_students = args.get(5).map_or(Ok(6), |a| a.parse())?;
    let args = Args {
        num_ideas,
        num_idea_gen,
        num_pkgs,
        num_pkg_gen,
        num_students,
    };

    hackathon(&args);
    Ok(())
}

fn per_thread_amount(thread_idx: usize, total: usize, threads: usize) -> usize {
    let per_thread = total / threads;
    let extras = total % threads;
    per_thread + (thread_idx < extras) as usize
}

fn hackathon(args: &Args) {
    // Use message-passing channel as event queue
    let (pkg_send, pkg_recv) = bounded::<PkgEvent>(args.num_pkgs);
    let (idea_send, idea_recv) = bounded::<IdeaEvent>(args.num_ideas + args.num_students);
    let mut idea_threads = vec![];
    let mut pkg_threads = vec![];
    let mut student_threads = vec![];

    // Spawn student threads
    for _i in 0..args.num_students {
        let mut student = Student::new(Sender::clone(&pkg_send), Receiver::clone(&pkg_recv), Receiver::clone(&idea_recv));
        let thread = spawn(move || student.run());
        student_threads.push(thread);
    }

    // Move file I/O outside of the threads to avoid loading more than once
    let pkgs: Vec<String> = fs::read_to_string("data/packages.txt").unwrap().lines().map(|s| s.to_owned()).collect();

    // Spawn package downloader threads. Packages are distributed evenly across threads.
    let mut start_idx = 0;
    for i in 0..args.num_pkg_gen {
        let num_pkgs = per_thread_amount(i, args.num_pkgs, args.num_pkg_gen);
        let downloader = PackageDownloader::new(pkgs.clone(), start_idx, num_pkgs, Sender::clone(&pkg_send));
        start_idx += num_pkgs;

        let thread = spawn(move || downloader.run());
        pkg_threads.push(thread);
    }
    assert_eq!(start_idx, args.num_pkgs);

    // Move file I/O outside of the threads to avoid loading more than once
    let products = fs::read_to_string("data/ideas-products.txt").unwrap();
    let customers = fs::read_to_string("data/ideas-customers.txt").unwrap();
    let ideas = cross_product(products, customers);

    // Spawn idea generator threads. Ideas and packages are distributed evenly across threads. In
    // each thread, packages are distributed evenly across ideas.
    let mut start_idx = 0;
    for i in 0..args.num_idea_gen {
        let num_ideas = per_thread_amount(i, args.num_ideas, args.num_idea_gen);
        let num_pkgs = per_thread_amount(i, args.num_pkgs, args.num_idea_gen);
        let num_students = per_thread_amount(i, args.num_students, args.num_idea_gen);
        let generator = IdeaGenerator::new(
            ideas.clone(),
            start_idx,
            num_ideas,
            num_students,
            num_pkgs,
            Sender::clone(&idea_send),
        );
        start_idx += num_ideas;

        let thread = spawn(move || generator.run());
        idea_threads.push(thread);
    }
    assert_eq!(start_idx, args.num_ideas);

    // Join all threads
    // Checksums of all the generated ideas and packages
    let mut idea_checksum = Checksum::default();
    let mut pkg_checksum = Checksum::default();
    for thread in idea_threads {
        let checksum = thread.join().unwrap();
        idea_checksum.update(checksum);
    }
    for thread in pkg_threads {
        let checksum = thread.join().unwrap();
        pkg_checksum.update(checksum);
    }
    // Checksums of the ideas and packages used by students to build ideas. Should match the
    // previous checksums.
    let mut student_idea_checksum = Checksum::default();
    let mut student_pkg_checksum = Checksum::default();
    for thread in student_threads {
        let checksum = thread.join().unwrap();
        student_idea_checksum.update(checksum.0);
        student_pkg_checksum.update(checksum.1);
    }

    println!("Global checksums:\nIdea Generator: {}\nStudent Idea: {}\nPackage Downloader: {}\nStudent Package: {}", 
    idea_checksum, student_idea_checksum, pkg_checksum, student_pkg_checksum);
}

// Idea names are generated from cross products between product names and customer names
fn cross_product(products: String, customers: String) -> Vec<String> {
    products
        .lines()
        .flat_map(|p| customers.lines().map(move |c| (format!("{} for {}", p.to_owned(), c.to_owned()))))
        .collect()
}