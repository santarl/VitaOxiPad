#include <assert.h>

#include <psp2/ctrl.h>
#include <psp2/motion.h>
#include <psp2/touch.h>

#include "ctrl.hpp"

NetProtocol::ButtonsData convert_pad_data(const SceCtrlData &data) {
  return NetProtocol::ButtonsData(
      (data.buttons & SCE_CTRL_SELECT) > 0, (data.buttons & SCE_CTRL_START) > 0,
      (data.buttons & SCE_CTRL_UP) > 0, (data.buttons & SCE_CTRL_RIGHT) > 0,
      (data.buttons & SCE_CTRL_DOWN) > 0, (data.buttons & SCE_CTRL_LEFT) > 0,
      (data.buttons & SCE_CTRL_LTRIGGER) > 0, (data.buttons & SCE_CTRL_RTRIGGER) > 0,
      (data.buttons & SCE_CTRL_TRIANGLE) > 0, (data.buttons & SCE_CTRL_CIRCLE) > 0,
      (data.buttons & SCE_CTRL_CROSS) > 0, (data.buttons & SCE_CTRL_SQUARE) > 0,
      (data.buttons & SCE_CTRL_VOLUP) > 0, (data.buttons & SCE_CTRL_VOLDOWN) > 0
    );
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

  sceCtrlPeekBufferPositive(0, &pad, 1);
  while (pad.timeStamp <= last_ts) {
    sceCtrlPeekBufferPositive(0, &pad, 1);
  }
  if (pad.buttons & SCE_CTRL_SELECT && pad.buttons & SCE_CTRL_LTRIGGER){
    sceKernelDelayThread(20 * 1000);
    pad.buttons |= SCE_CTRL_VOLDOWN;
  }
  if (pad.buttons & SCE_CTRL_SELECT && pad.buttons & SCE_CTRL_RTRIGGER){
    sceKernelDelayThread(20 * 1000);
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
