void setup() {
  // put your setup code here, to run once:
  pinMode(40, INPUT_PULLUP); // strobe
  pinMode(41, INPUT_PULLUP); // 0
  pinMode(42, INPUT_PULLUP); // 1
  pinMode(43, INPUT_PULLUP); // 2
  pinMode(44, INPUT_PULLUP); // 3
  pinMode(45, INPUT_PULLUP); // 4
  pinMode(46, INPUT_PULLUP); // 5
  pinMode(47, INPUT_PULLUP); // 6
  pinMode(48, INPUT_PULLUP); // 7
  digitalWrite(50, LOW);
  pinMode(50, OUTPUT); // busy
  digitalWrite(52, LOW);
  pinMode(52, OUTPUT); // gnd
  digitalWrite(52, LOW);
  pinMode(52, OUTPUT); // gnd
  Serial.begin(500000);
}

void loop() {
  // put your main code here, to run repeatedly:
  noInterrupts();
  while (PING & 1 << 1); // wait for strobe
  //delayMicroseconds(20);
  uint8_t PL = PINL;
  uint8_t PG = PING;
  uint8_t byte = !!(PG & 1 << 0) << 0 |
                 !!(PL & 1 << 7) << 1 |
                 !!(PL & 1 << 6) << 2 |
                 !!(PL & 1 << 5) << 3 |
                 !!(PL & 1 << 4) << 4 |
                 !!(PL & 1 << 3) << 5 |
                 !!(PL & 1 << 2) << 6 |
                 !!(PL & 1 << 1) << 7;
  while (!(PING & 1 << 1));
  //delayMicroseconds(1000);
  PORTB |= 1 << 3; // busy high
  interrupts();
  Serial.write(byte);
  //delayMicroseconds(10000);
  noInterrupts();
  PORTB &= ~(1 << 3); // busy low
}
