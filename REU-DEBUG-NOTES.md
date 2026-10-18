# REU Test Panic — Debug Notes (RESOLVED)

## Outcome

- **Failing assert:** `assert!(sum_x == 296)` in `test_reu_array()`
  (`demo/src/reu_tests.rs:146`).
- **Root cause:** `IndexMut::index_mut` in `reu/src/reu_array.rs` returned a
  `&mut T` into the cache **without calling `mark_dirty()`**. Writes via
  `array[i] = value` were lost on the next cache-window slide.
- **Fix:** added `self.mark_dirty();` in `index_mut` (one line + comment).
- **Verified:** rebuilt `cargo build -p demo --target mos-c64-none
  --features reu-hardware-tests`, ran in VICE — no panic, program reaches
  post-main idle loop; `println!("REU tests passed")` executed (any assert
  failure would freeze PC at the panic spin $20F2; PC was stably at $6252).

## Root-cause mechanism (windowed cache)

`REUArray` pages `window_size` elements between a RAM cache and REU memory.
`window_start_index` tracks the REU offset of the cached window; `dirty` says
whether the cache must be flushed (`prepare_slice` + `reu().push()`) before the
window slides. `ensure_in_cache(index, load)` does: if index outside window →
`flush()` → set `window_start = index` → optional `pull()`.

Every mutator marked dirty **except** `IndexMut::index_mut`:

| op         | marks dirty |
|------------|-------------|
| `push`     | yes         |
| `set`      | yes         |
| `with_mut` | yes         |
| `index_mut`| **no (bug)**|

Test sequence that triggered it (window_size = 10, 32 elements):
1. `array[7] = GameUnit{… x:70 …}` → `index_mut(7)` slides window to 7, loads
   REU[56..136] into cache, returns `&mut cache[0]`; caller writes x=70.
   `dirty` stays **false**.
2. `get(7)` twice: window still at 7 → reads the fresh cache value → PASS
   (masks the bug).
3. Iterator: `get(0)` slides window 7→0; `ensure_in_cache` calls `flush()`,
   but `dirty == false` → **no-op** → elem7 (x=70) never reaches REU.
   REU[56..64] keeps the old elem7 (x=3).
4. Iterator reaches index 7: window at 0, `cache[7]` = stale REU value →
   x=3 instead of 70.
5. `count == 32` passes; `sum_x` = 296 − 70 + 3 = **229** → panic.

## How the failing assertion was identified

Panic record chain in RAM (frozen VICE state, deterministic):
- Record at $CE8B: `8f 6c 3d` → msg_ptr **$6C8F**, byte `0x3d` = `len<<1|1`,
  len = 30.
- $6C8F = `"assertion failed: sum_x == 296"` (30 chars — exact match).

Panic record format (built by $0F2E, 4-byte records, allocated downward from
$02/$03): `[msg_lo, msg_hi, len<<1|1, ?]`. Assert trampoline at $1F54
(`A = len*2+1 → $06`). 19 assert call sites, all using convention
`LDA #len; LDX #$00; LDY #ptr_lo; STY $04; LDY #ptr_hi; STY $05; JSR $1F54`.

Assertion literal table (RAM addr → text):
$6A01 unit.number == index as u8 · $6A2D unit.speed == expected_speed ·
$6A5B unit.health == expected_health · $6A8B unit.x == expected_x ·
$6AB1 unit.y == expected_y · $6C8F sum_x == 296 (FAILED) · $6CAD count == 32 ·
$6CCA array.get(7).speed == 0xc · $6CF5 array.get(7).x == 70 ·
$6D1B array[31].y == 4 · $6D3D array[0].number == 0 · $6D63
array.get(10).speed == 3 · $6D8D array.get(25).x == 69 · $6DB4
array.get(5).x == 69 · $6DDA array.len() == 32 · $6DFD second.address() ==
first.address() + 256 · $6E38 third.address() == second.address() + 256 ·
$6E73 reused.address() == freed_address · $6EA6 value == 0xa5

Other strings: $6AD7 REUArray index out of bounds · $6B00 REUArray capacity
exceeded · $6B51/$6B56 panic template · $6B5D hex digit table · $6C25/$6C41/
$6C62 overflow messages · $6ED1 REU address exceeds 24 bits · $6EEE REU DMA
range exceeds 16 MiB · $6EFC initialize REU allocator first · $6F1E out of REU
memory · $6F39 REU chunk exceeds C64 DMA address space.

## Debugging artifacts & recipes

- File→RAM mapping: **`file[i] ↔ RAM[$07FF+i]`** (non-standard PRG header,
  base $07FF not $0801).
- Disassembler: `/tmp/d65.py <start_ram>,<count>` (run from /tmp; desyncs in
  data-heavy regions — use raw byte dumps for precision).
- Panic spin: $20F0 `ADC $0F` / $20F2 `JMP $20F0`; frozen PC was $20F2 pre-fix.
- Freeze state pre-fix: PC=$20F2, SP=EA, $02/$03=$CE6A (record alloc ptr),
  $08/$09=$6B51 (template ptr). $CEE0+ filled 0xA5 (REU guard/DMA artifact).
- VICE monitor: `m <start> <end>` takes TWO addresses, not a count. One query
  per fresh `ncat 127.0.0.1 6510` connection. Large dumps truncate — read in
  ≤256-byte chunks.
- VICE launch (working): Xvfb :99 @ 1024x768x24;
  `x64sc -reu -reusize 8192 -remotemonitor -autostart /tmp/demo.prg
  -kernal/-basic/-chargen <stock ROMs> -logfile /tmp/vice.log`;
  wait for port 6510. Flaky: restart Xvfb + VICE on `malloc(): unaligned
  tcache chunk` crashes. No `-ice` option in VICE 3.10.
- Success signal: PC stably at post-main idle ($6252 for this build),
  stopwatch advancing, screen shows test output (e.g. `Unit at x=69: (5,11)`).
- PETSCII note: screen RAM $0400, 25×40; upper/lower case codes are swapped vs
  ASCII ($41-$5A = a-z, $61-$7A = A-Z).

## Remaining open question (secondary, not blocking)

Why C64 RAM $CEE0+ contained a 0xA5 fill in the pre-fix run (REU guard/DMA
artifact). Did not recur as a problem post-fix; revisit only if it reappears.
