use blist::*;

fn main() {
	let mut list = BList::<i32, 15>::new();

	for i in -50..50 {
		list.insert(i);
	}

	for i in -5..15 {
		list.insert(i);
	}

	for _ in 0..50 {
		list.insert(2);
	}

	println!("{:#?}", list);
}
