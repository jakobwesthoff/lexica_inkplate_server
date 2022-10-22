#pragma once

#include "util.h"
#include "Inkplate.h"

// #define ALWAYS_SHOW_BATTERY
#define BATTERY_WARNING_LEVEL 3.8

void ALWAYS_INLINE checkBattery(Inkplate *display)
{
  double batteryLevel = display->readBattery();
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
