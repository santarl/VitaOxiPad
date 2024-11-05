#ifndef EVENTS_HPP
#define EVENTS_HPP

enum MainEvent {
  // For NetThread
  PC_DISCONNECT          = 1 << 0,  // 0x0001
  PC_CONNECT             = 1 << 1,  // 0x0002
  NET_CONNECT            = 1 << 2,  // 0x0004
  NET_DISCONNECT         = 1 << 3,  // 0x0008
  
  // For StatusThread
  BATTERY_LEVEL          = 1 << 8,  // 0x0100
  STATUS_CHARGER         = 1 << 9,  // 0x0200
  WIFI_SIGNAL            = 1 << 10, // 0x0400
};

#endif // EVENTS_HPP
