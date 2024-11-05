#ifndef STATUS_HPP
#define STATUS_HPP

#include <atomic>
#include <mutex>

#include <psp2/net/net.h>
#include <psp2/net/netctl.h>
#include <psp2/kernel/threadmgr.h>

#include "events.hpp"

struct StatusSharedData {
    std::atomic<uint32_t> events;
    int battery_level = 0;
    bool charger_connected = false;
    unsigned int wifi_signal_strength = 0;
    std::mutex mutex;
};

typedef struct {
    SceUID event_flag;
    StatusSharedData* shared_data;
} StatusThreadMessage;

int status_thread(unsigned int arglen, void* argp);

#endif // STATUS_HPP
