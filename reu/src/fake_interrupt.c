void fake_irq_function() {
  asm volatile("rti\n");
}