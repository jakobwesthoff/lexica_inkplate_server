#pragma once

#include "util.h"
#include "Inkplate.h"

//#define ALWAYS_SHOW_BATTERY
#define BATTERY_WARNING_LEVEL 3.6

#ifdef ARDUINO_INKPLATECOLOR
// Currently needed as the new port multiplexer is not supported by the default library yet:
// https://github.com/SolderedElectronics/Inkplate-Arduino-library/issues/169#issuecomment-1331716568
// Combined with the following open PR: https://github.com/SolderedElectronics/Inkplate-Arduino-library/pull/171/commits/124462fdf49963d7227881cc4a28c28f4ff40f6e
void pcal6416ModifyReg(uint8_t _reg, uint8_t _bit, uint8_t _state)
{
  uint8_t reg;
  uint8_t mask;
  const uint8_t pcalAddress = 0b00100000;

  Wire.beginTransmission(pcalAddress);
  Wire.write(_reg);
  Wire.endTransmission();

  Wire.requestFrom(pcalAddress, (uint8_t)1);
  reg = Wire.read();

  mask = 1 << _bit;
  reg = ((reg & ~mask) | (_state << _bit));

  Wire.beginTransmission(pcalAddress);
  Wire.write(_reg);
  Wire.write(reg);
  Wire.endTransmission();
}

double readBatteryVoltage()
{
  // Set PCAL P1-1 to output. Do a ready-modify-write operation.
  pcal6416ModifyReg(0x07, 1, 0);

  // Set pin P1-1 to the high -> enable MOSFET for battrey voltage measurement.
  pcal6416ModifyReg(0x03, 1, 1);

  // Wait a little bit
  delay(5);

  // Read analog voltage. Battery measurement is connected to the GPIO35 on the ESP32.
  uint32_t batt_mv = analogReadMilliVolts(35);

  // Turn off the MOSFET.
  pcal6416ModifyReg(0x03, 1, 0);

  // Calculate the voltage
  return (double(batt_mv) / 1000 * 2);
}
#endif

void ALWAYS_INLINE checkBattery(Inkplate *display)
{
#ifdef ARDUINO_INKPLATECOLOR
  double batteryLevel = readBatteryVoltage();
#else
  double batteryLevel = display->readBattery();
#endif
  Serial.print("Battery level: ");
  Serial.println(batteryLevel);
#ifndef ALWAYS_SHOW_BATTERY
  if (batteryLevel != 0.0 && batteryLevel < BATTERY_WARNING_LEVEL)
  {
#endif
    display->setTextColor(7, 0);
    display->setCursor(0, E_INK_HEIGHT - 25);
#ifdef ALWAYS_SHOW_BATTERY
    display->print("Battery level: ");
#else
  display->print("Battery level low! (");
#endif
    display->print(batteryLevel);
    display->print(" V");
#ifndef ALWAYS_SHOW_BATTERY
    display->println(")");
#endif
#ifndef ALWAYS_SHOW_BATTERY
  }
#endif
}
