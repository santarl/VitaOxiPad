#include <assert.h>

#include <psp2/ctrl.h>
#include <psp2/libdbg.h>
#include <psp2/motion.h>
#include <psp2/touch.h>

#include <ctrl.hpp>
#include "kctrl-kernel.h"

NetProtocol::ButtonsData convert_pad_data(const SceCtrlData &data) {
  return NetProtocol::ButtonsData(
      (data.buttons & SCE_CTRL_SELECT) > 0, (data.buttons & SCE_CTRL_START) > 0,
      (data.buttons & SCE_CTRL_UP) > 0, (data.buttons & SCE_CTRL_RIGHT) > 0,
      (data.buttons & SCE_CTRL_DOWN) > 0, (data.buttons & SCE_CTRL_LEFT) > 0,
      (data.buttons & SCE_CTRL_LTRIGGER) > 0, (data.buttons & SCE_CTRL_RTRIGGER) > 0,
      (data.buttons & SCE_CTRL_TRIANGLE) > 0, (data.buttons & SCE_CTRL_CIRCLE) > 0,
      (data.buttons & SCE_CTRL_CROSS) > 0, (data.buttons & SCE_CTRL_SQUARE) > 0,
      (data.buttons & SCE_CTRL_VOLUP) > 0, (data.buttons & SCE_CTRL_VOLDOWN) > 0);
}

flatbuffers::Offset<NetProtocol::TouchData>
convert_touch_data(flatbuffers::FlatBufferBuilder &builder, const SceTouchData &data) {
  std::vector<NetProtocol::TouchReport> reports;
  reports.reserve(data.reportNum);
  std::transform(data.report, data.report + data.reportNum, std::back_inserter(reports),
                 [](const SceTouchReport &report) {
                   return NetProtocol::TouchReport(report.force, report.id, report.x, report.y);
                 });
  return NetProtocol::CreateTouchDataDirect(builder, &reports);
}

void get_ctrl_as_netprotocol(flatbuffers::FlatBufferBuilder &builder, SharedData *shared_data) {
  builder.Clear();
  static uint64_t last_ts = 1024;
  SceCtrlData pad;
  SceMotionState motion_data;
  SceTouchData touch_data_front, touch_data_back;

  int res = kctrlGetCtrlData(0, &pad, 1);
  if (res < 0) {
    // Handle error
    SCE_DBG_LOG_ERROR("kctrlGetCtrlData failed: 0x%08X", res);
    return;
  }
  while (pad.timeStamp <= last_ts) {
    kctrlGetCtrlData(0, &pad, 1);
  }

  // if (pad.buttons & SCE_CTRL_UP)
  //   SCE_DBG_LOG_DEBUG("SCE_CTRL_UP");
  // if (pad.buttons & SCE_CTRL_DOWN)
  //   SCE_DBG_LOG_DEBUG("SCE_CTRL_DOWN");
  // if (pad.buttons & SCE_CTRL_LEFT)
  //   SCE_DBG_LOG_DEBUG("SCE_CTRL_LEFT");
  // if (pad.buttons & SCE_CTRL_RIGHT)
  //   SCE_DBG_LOG_DEBUG("SCE_CTRL_RIGHT");
  // if (pad.buttons & SCE_CTRL_LTRIGGER)
  //   SCE_DBG_LOG_DEBUG("SCE_CTRL_LTRIGGER");
  // if (pad.buttons & SCE_CTRL_RTRIGGER)
  //   SCE_DBG_LOG_DEBUG("SCE_CTRL_RTRIGGER");
  // if (pad.buttons & SCE_CTRL_TRIANGLE)
  //   SCE_DBG_LOG_DEBUG("SCE_CTRL_TRIANGLE");
  // if (pad.buttons & SCE_CTRL_CIRCLE)
  //   SCE_DBG_LOG_DEBUG("SCE_CTRL_CIRCLE");
  // if (pad.buttons & SCE_CTRL_CROSS)
  //   SCE_DBG_LOG_DEBUG("SCE_CTRL_CROSS");
  // if (pad.buttons & SCE_CTRL_SQUARE)
  //   SCE_DBG_LOG_DEBUG("SCE_CTRL_SQUARE");
  // if (pad.buttons & SCE_CTRL_START)
  //   SCE_DBG_LOG_DEBUG("SCE_CTRL_START");
  // if (pad.buttons & SCE_CTRL_SELECT)
  //   SCE_DBG_LOG_DEBUG("SCE_CTRL_SELECT");
  // if (pad.buttons & SCE_CTRL_PSBUTTON)
  //   SCE_DBG_LOG_DEBUG("SCE_CTRL_PSBUTTON");
  // if (pad.buttons & SCE_CTRL_VOLUP)
  //   SCE_DBG_LOG_DEBUG("SCE_CTRL_VOLUP");
  // if (pad.buttons & SCE_CTRL_VOLDOWN)
  //   SCE_DBG_LOG_DEBUG("SCE_CTRL_VOLDOWN");


  if (pad.buttons & SCE_CTRL_SELECT && pad.buttons & SCE_CTRL_LTRIGGER){
    sceKernelDelayThread(50 * 1000);
    pad.buttons |= SCE_CTRL_VOLDOWN;
  }
  if (pad.buttons & SCE_CTRL_SELECT && pad.buttons & SCE_CTRL_RTRIGGER){
    sceKernelDelayThread(50 * 1000);
    pad.buttons |= SCE_CTRL_VOLUP;
  }

  auto buttons = convert_pad_data(pad);

  sceTouchPeek(SCE_TOUCH_PORT_FRONT, &touch_data_front, 1);
  auto data_front = convert_touch_data(builder, touch_data_front);

  sceTouchPeek(SCE_TOUCH_PORT_BACK, &touch_data_back, 1);
  auto data_back = convert_touch_data(builder, touch_data_back);

  sceMotionGetState(&motion_data);
  NetProtocol::Vector3 accel(motion_data.acceleration.x, motion_data.acceleration.y,
                             motion_data.acceleration.z);
  NetProtocol::Vector3 gyro(motion_data.angularVelocity.x, motion_data.angularVelocity.y,
                            motion_data.angularVelocity.z);
  NetProtocol::MotionData motion(gyro, accel);

  int charge_percent = shared_data->battery_level;

  auto content =
      NetProtocol::CreatePad(builder, &buttons, pad.lx, pad.ly, pad.rx, pad.ry, data_front,
                             data_back, &motion, pad.timeStamp, charge_percent);

  auto packet =
      NetProtocol::CreatePacket(builder, NetProtocol::PacketContent::Pad, content.Union());
  builder.FinishSizePrefixed(packet);
  last_ts = pad.timeStamp;
}
