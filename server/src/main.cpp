#include <arpa/inet.h>
#include <psp2/ctrl.h>
#include <psp2/kernel/threadmgr.h>
#include <psp2/libdbg.h>
#include <psp2/motion.h>
#include <psp2/power.h>
#include <psp2/sysmodule.h>
#include <psp2/touch.h>
#include <vita2d.h>

#include "ctrl.hpp"
#include "events.hpp"
#include "net.hpp"
#include "status.hpp"

#include <common.h>

constexpr size_t NET_INIT_SIZE = 1 * 1024 * 1024;

vita2d_pgf *debug_font;

std::atomic<bool> g_net_thread_running(true);
std::atomic<bool> g_status_thread_running(true);

int stop_thread(SceUID thread_uid, SceUInt timeout) {
  int wait_result = sceKernelWaitThreadEnd(thread_uid, NULL, &timeout);
  if (wait_result < 0) {
    SCE_DBG_LOG_ERROR("Error waiting for thread to end: 0x%08X", wait_result);
  }
  int delete_result = sceKernelDeleteThread(thread_uid);
  if (delete_result < 0) {
    SCE_DBG_LOG_ERROR("Error deleting thread: 0x%08X", delete_result);
  }
  SCE_DBG_LOG_TRACE("Thread stopped and deleted successfully");
  return delete_result;
}

int main() {
  // Enabling analog, motion and touch support
  sceCtrlSetSamplingMode(SCE_CTRL_MODE_ANALOG_WIDE);
  sceMotionStartSampling();
  sceTouchSetSamplingState(SCE_TOUCH_PORT_FRONT, SCE_TOUCH_SAMPLING_STATE_START);
  sceTouchSetSamplingState(SCE_TOUCH_PORT_BACK, SCE_TOUCH_SAMPLING_STATE_START);
  sceTouchEnableTouchForce(SCE_TOUCH_PORT_FRONT);
  sceTouchEnableTouchForce(SCE_TOUCH_PORT_BACK);

  // Motion stuffs
  sceMotionSetGyroBiasCorrection(1);
  sceMotionSetTiltCorrection(1);
  sceMotionSetDeadband(0);

  // Reduce CPU and GPU frequency to save battery
  scePowerSetArmClockFrequency(41);
  scePowerSetBusClockFrequency(55);
  scePowerSetGpuClockFrequency(41);
  scePowerSetGpuXbarClockFrequency(83);

  // Initializing graphics stuffs
  vita2d_init();
  vita2d_set_clear_color(RGBA8(0x00, 0x00, 0x00, 0xFF));
  debug_font = vita2d_load_default_pgf();
  uint32_t need_color = 0;
  uint32_t common_color = RGBA8(0xFF, 0xFF, 0xFF, 0xFF); // White color
  uint32_t error_color = RGBA8(0xFF, 0x00, 0x00, 0xFF);  // Bright red color
  uint32_t done_color = RGBA8(0x00, 0xFF, 0x00, 0xFF);   // Bright green color

  // Initializing network stuffs
  sceSysmoduleLoadModule(SCE_SYSMODULE_NET);
  char vita_ip[INET_ADDRSTRLEN];
  std::vector<uint8_t> net_init_memory(NET_INIT_SIZE);
  SceNetInitParam initparam = {net_init_memory.data(), NET_INIT_SIZE, 0};
  int ret = sceNetShowNetstat();
  if ((unsigned)ret == SCE_NET_ERROR_ENOTINIT) {
    ret = sceNetInit(&initparam);
    if (ret < 0) {
      SCE_DBG_LOG_ERROR("Network initialization failed: %s", sce_net_strerror(ret));
      return -1;
    }
  }

  sceNetCtlInit();
  SceNetCtlInfo info;

  SharedData shared_data;
  SceUID ev_flag = sceKernelCreateEventFlag("main_event_flag", 0, 0, NULL);
  shared_data.events = 0;
  shared_data.battery_level = scePowerGetBatteryLifePercent();
  shared_data.charger_connected = scePowerIsBatteryCharging();
  ThreadMessage message = {ev_flag, &shared_data};

  // Creating events and status thread
  SceUID status_thread_uid = sceKernelCreateThread("StatusThread", status_thread, 0x10000100,
                                                   0x10000, 0, SCE_KERNEL_CPU_MASK_USER_1, NULL);
  sceKernelStartThread(status_thread_uid, sizeof(ThreadMessage), &message);

  // Creating events and network thread
  SceUID net_thread_uid = sceKernelCreateThread("NetThread", &net_thread, 0x10000100, 0x10000, 0,
                                                SCE_KERNEL_CPU_MASK_USER_2, nullptr);
  if (net_thread_uid < 0) {
    SCE_DBG_LOG_ERROR("Error creating thread: 0x%08X", net_thread_uid);
    return -1;
  }
  sceKernelStartThread(net_thread_uid, sizeof(ThreadMessage), &message);

  uint32_t events;
  sceNetCtlInetGetState(reinterpret_cast<int *>(&events));
  bool connected_to_network = events & SCE_NETCTL_STATE_CONNECTED;
  bool pc_connect_state = false;
  if (connected_to_network) {
    sceNetCtlInetGetInfo(SCE_NETCTL_INFO_GET_IP_ADDRESS, &info);
    snprintf(vita_ip, INET_ADDRSTRLEN, "%s", info.ip_address);
  }
  events = 0;

  // Main loop for events
  // Loop is executed if the MainEvent state changes
  do {
    vita2d_start_drawing();
    vita2d_clear_screen();
    vita2d_pgf_draw_text(debug_font, 2, 20, common_color, 1.0,
                         "VitaOxiPad v1.1.0 \nbuild " __DATE__ ", " __TIME__);

    if (events & MainEvent::NET_CONNECT) {
      connected_to_network = true;
      sceNetCtlInetGetInfo(SCE_NETCTL_INFO_GET_IP_ADDRESS, &info);
      snprintf(vita_ip, INET_ADDRSTRLEN, "%s", info.ip_address);
    } else if (events & MainEvent::NET_DISCONNECT) {
      connected_to_network = false;
    }

    if (connected_to_network) {
      vita2d_pgf_draw_textf(debug_font, 750, 20, common_color, 1.0,
                            "Listening on:\nIP: %s\nPort: %d", vita_ip, NET_PORT);
    } else {
      vita2d_pgf_draw_text(debug_font, 750, 20, error_color, 1.0, "Not connected\nto a network :(");
    }

    if (events & MainEvent::PC_CONNECT) {
      pc_connect_state = true;
    } else if (events & MainEvent::PC_DISCONNECT) {
      pc_connect_state = false;
    }
    if (pc_connect_state) {
      vita2d_pgf_draw_textf(debug_font, 2, 540, done_color, 1.0, "Status: Connected (%s)",
                            shared_data.client_ip);
    } else {
      vita2d_pgf_draw_text(debug_font, 2, 540, error_color, 1.0, "Status: Not connected :(");
    }

    if (shared_data.charger_connected) {
      need_color = done_color;
    } else if (shared_data.battery_level < 30) {
      need_color = error_color;
    } else {
      need_color = common_color;
    }
    vita2d_pgf_draw_textf(debug_font, 785, 520, need_color, 1.0, "Battery: %s%d%%",
                          shared_data.charger_connected ? "+" : "", shared_data.battery_level);

    if (shared_data.wifi_signal_strength < 50) {
      need_color = error_color;
    } else {
      need_color = common_color;
    }
    vita2d_pgf_draw_textf(debug_font, 785, 540, need_color, 1.0, "WiFi signal: %d%%",
                          shared_data.wifi_signal_strength);

    vita2d_end_drawing();
    vita2d_wait_rendering_done();
    vita2d_swap_buffers();
  } while (sceKernelWaitEventFlag(ev_flag, 0xFFFFFFFF, SCE_EVENT_WAITOR | SCE_EVENT_WAITCLEAR,
                                  &events, NULL) == 0);

  // Turn on network thread stop signal and wait for its normal termination
  SceUInt THREAD_TIMEOUT = (SceUInt)(15 * 1000 * 1000);
  g_net_thread_running.store(false);
  g_status_thread_running.store(false);
  sceKernelSetEventFlag(ev_flag, MainEvent::NET_DISCONNECT);
  SCE_DBG_LOG_TRACE("StatusThread stop...");
  stop_thread(status_thread_uid, THREAD_TIMEOUT);
  SCE_DBG_LOG_TRACE("NetThread stop...");
  stop_thread(net_thread_uid, THREAD_TIMEOUT);

  sceNetCtlTerm();
  sceNetTerm();
  sceSysmoduleUnloadModule(SCE_SYSMODULE_NET);

  vita2d_fini();
  vita2d_free_pgf(debug_font);
  return 1;
}
