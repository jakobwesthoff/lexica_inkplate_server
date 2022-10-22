// Configure based on your setup
// After that comment out the following line:
#error Configure your WiFi credentials in 'wifi_config.h'

// Enable to create own access point otherwise the defined one will be joined.
// #define SOFTAP_MODE

// Conversion factor for micro seconds to seconds
#define uS_TO_S_FACTOR 1000000

void goto_sleep(uint64_t);

#include "util.h"

#ifndef SOFTAP_MODE
const char *ssid = "<ADD SSID HERE>";
const char *password = "<ADD AP PASSWORD HERE>";

ALWAYS_INLINE void initWiFi()
{
  WiFi.mode(WIFI_STA);
  WiFi.begin(ssid, password);
  log_d("Connecting to WiFi with SSID %s and password %s", ssid, password);
  uint32_t wait_counter = 0;
  uint32_t wait_delay = 250;
  uint32_t wait_sleep_s = 300;
  while (WiFi.status() != WL_CONNECTED)
  {
    if (wait_counter * wait_delay > 5000 ) {
      log_d("Wifi connection failed after %d ms", wait_counter * wait_delay);
      log_d("Going to sleep for 5 minutes", wait_counter * wait_delay);
      goto_sleep(wait_sleep_s * uS_TO_S_FACTOR);
    }
    log_d("Waiting for connection...");
    delay(wait_delay);
    wait_counter++;
  }
  log_d("Connected to wifi with ip address %s", WiFi.localIP().toString().c_str());
}

#endif

#ifdef SOFTAP_MODE
const char *ssid = "ESP32-Access-Point";

ALWAYS_INLINE void initWiFi()
{
  log_d("Setting up WiFi AP with SSID %s", ssid);
  WiFi.softAP(ssid);
  IPAddress IP = WiFi.softAPIP();
  log_d("Established access point with SSID %s and ip address %s", ssid, IP.toString().c_str());
}
#endif
