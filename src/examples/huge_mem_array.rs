
struct GameUnit {
    speed: u8,
    health: u8,
    x: u16,
    y: u16,
}

fn reu_array_playground() {
    // Allocate 100,000 GameUnit elements plus a cache that can hold 10 elements at a time
    let mut array = REUArray::<GameUnit>::new(100000, 10);

    println!("Setting Unit 1");
    array[1]=GameUnit { speed: 1, health: 2, x: 3, y: 4 };
    println!("Setting Unit 8");
    array[8]=GameUnit { speed: 3, health: 20, x: 10, y: 20 };

    println!("Getting health at index 1: {}", array[1].health);

    println!("Setting x of Element 70000 to 100");
    array[70000].x = 100;
    println!("Getting speed at index 8: {}", array[8].speed);
    println!("Getting y at index 70000: {}", array[70000].y);

    // find all ead units
    let dead_units = array.into_iter().filter(|unit| *unit.health == 0);

    // print coords of dead units
    for u in dead_units {
        println!("Died at: ({},{})", *u.x, u.y);
    }
}