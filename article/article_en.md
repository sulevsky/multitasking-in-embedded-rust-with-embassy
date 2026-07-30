# Async Rust and Embedded Systems with Embassy

## 1. Problem statement 
Let's have a simple example of having IO operation very common in embedded development
Say we have a simple setup STM32[TODO] MCU and XXXX[TODO] RTC and we want to read clock values with I2C. Quite simple setup, but it will be sufficient to illustrate our ideas and not add unnecessary complications.

## What are our options to read RTC values via I2C.

### 1. Simple as hell - Blocking read
- describe blocking read
- add example from [code](../src/bin/blocking.rs)
- describe pros and cons
  - [incorporate] sleep is antipattern but why it's used so often, because it's useful and easy to understand, what if we could use it, but without downsides

### 2. Interrupts - No blocking
- describbe reading with interrupts
- show code example
- pros and cons
  - visualize logic is scatered

### 3. DMA with interrupts
- describbe reading with DMA and interrupts
- show code example
- pros and cons
  - not easy to reason about program logic is scattered in many places
  - hard to setup
  - effective from perspective of execution

## But can we take best from both sides - easiness to understand from stringht blocking flow and execution effectiveness from DMA/Interrupts
	1. We can take ADC and easily setup DMA
	2. And with async this code will be simple to understand
	3. and with async this code will be efficient

- Why `async` matters
- Why `async` matters in embedded
- Embassy code example
- Setup
- Task
- Testing
- [TODO] add a reference to the importance of testing article

## Summary 
- what we've achieved is the simple and effective code

## Resources
1. Rust book https://doc.rust-lang.org/book/ch17-00-async-await.html
1. Embassy book https://embassy.dev/book/


-----------------

## TODO:
- [ ] async book
- [ ] embassy
	- [ ] https://embassy.dev/book/
- [ ] https://learn.flowresearch.tech/curriculum/rust-engineering/async-rust-and-tokio
- [ ] rename title and repo name due to collision with a talk name
 - [ ] [dma] hard to setup, has benefits of taking load from MCU, fits well in async model
 `However, because DMA is more complex to set-up, it is less widely used in the embedded community. Embassy aims to change that by making DMA the first choice rather than the last.` https://embassy.dev/book/#_introduction
 - [ ] [timer] easier to setup
 - [ ] investigate async-hal and async-hal-io
 - [ ] reference to a prev article when describing "why rust" and "how to test"
 - [ ] is it easy to test (1. dma , 2. async)
 - [ ] sleep mode to save power
- [ ] consider drawing a diagram cooperative multitaskings 
 

## DONE
- [x] rust book
	- https://doc.rust-lang.org/book/ch17-00-async-await.html
- [x] Async Rust in Embedded Systems with Embassy - Dario Nieuwenhuis https://www.youtube.com/watch?v=H7NtzyP9q8E



