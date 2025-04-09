mod auth;
mod fetch;
mod parse;

use parse::parse_arguments;
use fetch::version::Version;

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let _ = parse_arguments();

	let version_id = String::from("1.14");

	let version_1_12 = Version::new(&version_id).await;

	version_1_12.unwrap().update().await;

	// let racer01 = F1Racer::new();
	// let mut racer02 = F1Racer::new();

	// racer02.lap_times.pop();
	// racer02.lap_times.push(42);
	
	// let handle01 = tokio::task::spawn(racer01);
	// let handle02 = tokio::task::spawn(racer02);
	
	// loop {
	// 	if handle01.is_finished() && handle02.is_finished() {
	// 		break;
	// 	}
	// 	std::thread::sleep(std::time::Duration::from_millis(300));
	// }

	Ok(())
}

struct F1Racer {
	name: String,
	completed_laps: u8,
	laps: u8,
	best_lap_time: u8,
	lap_times: Vec<u8>,
}

impl F1Racer {
	fn new() -> F1Racer {
		F1Racer {
			name: "Zigmut".to_string(),
			laps: 5,
			completed_laps: 0,
			best_lap_time: 255,
			lap_times: vec![87u8, 64, 126, 95, 75]
		}
	}

	fn do_lap(&mut self) {
		println!("{} does a real shit!", self.name);
		let lap_time = self.lap_times.pop();

		if lap_time.is_some() && lap_time.unwrap() < self.best_lap_time {
			self.best_lap_time = lap_time.unwrap();
		}

		self.completed_laps += 1;
	}
}

impl std::future::Future for F1Racer {
	type Output = u8;
	fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
		println!("Thread assigned id is {:?}", std::thread::current().id());
		if self.completed_laps < self.laps {
			self.get_mut().do_lap();
			cx.waker().wake_by_ref();
			return std::task::Poll::Pending;
		}
		println!("{} smashed 'em all!", self.name);
		println!("{}'s Best lap: {}", self.name, self.best_lap_time);;
		std::task::Poll::Ready(self.best_lap_time)
	}
}