#ifndef STATUS_HPP
#define STATUS_HPP

#include <atomic>
#include <mutex>

#include <psp2/kernel/threadmgr.h>
#include <psp2/net/net.h>
#include <psp2/net/netctl.h>

#include <atomic>

#include "events.hpp"

extern std::atomic<bool> g_status_thread_running;

int status_thread(unsigned int arglen, void *argp);

#endif // STATUS_HPP
