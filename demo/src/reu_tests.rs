use mos_hardware::c64::sid;
use reu::ram_expansion_unit;
use reu::REUArray;
use ufmt_stdio::*; // stdio dla środowisk, które nie mają std

pub fn alloc_test() {
    sid().start_random_generator();
    let chunk_count = sid().rand8(20);

    let reu = ram_expansion_unit::reu();
    reu.init_allocator(8 * 1024 * 1024);

    for _ in 0..chunk_count {
        let size = (sid().rand16(1000) * 16 + 1) as u32;
        let chunk = reu.alloc(size);
        println!("got reu chunk: {:?}!", chunk);
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct GameUnit {
    number: u8,
    speed: u8,
    health: u8,
    padding: u8,
    x: u16,
    y: u16,
}

fn assert_guard_filled(bytes: &[u8; 256]) {
    for index in 0..bytes.len() {
        let value = unsafe { core::ptr::read_volatile(bytes.as_ptr().add(index)) };
        assert!(value == 0xa5);
    }
}

pub fn test_reu_array() {
    let reu = ram_expansion_unit::reu();
    reu.init_allocator(8 * 1024 * 1024);

    assert!(core::mem::size_of::<GameUnit>() * 32 == 256);

    // Exact block alignment makes a partial final window adjacent to the guard chunk.
    let mut array = REUArray::<GameUnit>::with_capacity(32, 10);
    let guard = reu.alloc(256);
    reu.fill_reu(guard.address(), 256, 0xa5);
    let mut guard_data = [0; 256];
    guard.pull(reu, guard_data.as_mut_ptr() as usize);
    assert_guard_filled(&guard_data);

    for i in 0..32 {
        array.push(GameUnit {
            number: i as u8,
            speed: 1,
            health: 2,
            padding: 0,
            x: 3,
            y: 4,
        });
    }

    assert!(array.len() == 32);
    assert!(!array.is_empty());

    array.with_mut(5, |unit| {
        unit.speed = 0xa;
        unit.health = 0xb;
        unit.x = 69;
        unit.y = 11;
    });
    array.with_mut(25, |unit| unit.x = 69);
    array.set(
        10,
        GameUnit {
            number: 10,
            speed: 2,
            health: 3,
            padding: 0,
            x: 4,
            y: 5,
        },
    );
    array.for_each_mut(|_, unit| unit.speed = unit.speed.wrapping_add(1));

    assert!(array.get(5).x == 69);
    assert!(array.get(25).x == 69);
    assert!(array.get(10).speed == 3);

    array.for_each(|index, unit| {
        let expected_speed = match index {
            5 => 0xb,
            10 => 3,
            _ => 2,
        };
        let expected_health = if index == 5 { 0xb } else if index == 10 { 3 } else { 2 };
        let expected_x = if index == 5 || index == 25 {
            69
        } else if index == 10 {
            4
        } else {
            3
        };
        let expected_y = if index == 5 {
            11
        } else if index == 10 {
            5
        } else {
            4
        };

        assert!(unit.number == index as u8);
        assert!(unit.speed == expected_speed);
        assert!(unit.health == expected_health);
        assert!(unit.x == expected_x);
        assert!(unit.y == expected_y);
        if unit.x == 69 {
            println!("{} Unit at x=69: ({},{})", unit.number, unit.x, unit.y);
        }
    });

    // Index / IndexMut coverage.
    assert!(array[0].number == 0);
    assert!(array[31].y == 4);
    array[7] = GameUnit {
        number: 7,
        speed: 0xc,
        health: 0xd,
        padding: 0,
        x: 70,
        y: 12,
    };
    assert!(array.get(7).x == 70);
    assert!(array.get(7).speed == 0xc);

    // Iteration coverage.
    let mut sum_x = 0u32;
    let mut count = 0u32;
    for unit in &mut array {
        sum_x += unit.x as u32;
        count += 1;
    }
    assert!(count == 32);
    // 0..=6 + 8..=9 + 11..=31 carry x=3, 10 carries x=4, 5/25 carry x=69, 7 carries 70.
    // sum_x = 3*28 + 4 + 2*69 + 70 = 84 + 4 + 138 + 70 = 296.
    assert!(sum_x == 296);

    drop(array);

    guard.pull(reu, guard_data.as_mut_ptr() as usize);
    assert_guard_filled(&guard_data);
}

pub fn test_reu_allocator() {
    let reu = ram_expansion_unit::reu();
    reu.init_allocator(8 * 1024 * 1024);

    let first = reu.alloc(1);
    let second = reu.alloc(256);
    let third = reu.alloc(257);
    assert!(second.address() == first.address() + 256);
    assert!(third.address() == second.address() + 256);

    let freed_address = second.address();
    drop(second);

    let reused = reu.alloc(1);
    assert!(reused.address() == freed_address);

    drop(reused);
    drop(first);
    drop(third);
}
