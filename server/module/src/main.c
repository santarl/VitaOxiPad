#include <psp2/appmgr.h>
#include <psp2/ctrl.h>
#include <psp2/io/devctl.h>
#include <psp2/io/dirent.h>
#include <psp2/io/fcntl.h>
#include <psp2/io/stat.h>
#include <psp2/kernel/modulemgr.h>
#include <psp2/kernel/processmgr.h>
#include <psp2/power.h>
#include <psp2/shellutil.h>
#include <psp2/vshbridge.h>
#include <psp2kern/ctrl.h>
#include <psp2kern/kernel/cpu.h>
#include <psp2kern/kernel/debug.h>
#include <psp2kern/kernel/modulemgr.h>
#include <psp2kern/kernel/sysmem.h>
#include <psp2kern/kernel/threadmgr.h>
#include <taihen.h>

#include <malloc.h>
#include <stdarg.h>
#include <stdio.h>
#include <string.h>

#include "kctrl-kernel.h"

int kctrlGetCtrlData(int port, SceCtrlData *pad_data, int count) {
  SceCtrlData pad;

  uint32_t state;
  ENTER_SYSCALL(state);

  int res = ksceCtrlReadBufferPositive(port, &pad, count);
  ksceKernelMemcpyKernelToUser((uintptr_t)pad_data, &pad, sizeof(SceCtrlData));

  EXIT_SYSCALL(state);
  return res;
}

void _start() __attribute__((weak, alias("module_start")));
int module_start(SceSize args, const void *argp) { return SCE_KERNEL_START_SUCCESS; }

void _stop() __attribute__((weak, alias("module_stop")));
int module_stop(SceSize args, const void *argp) { return SCE_KERNEL_STOP_SUCCESS; }
