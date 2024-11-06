#ifndef __CTRL_H__
#define __CTRL_H__

#include <netprotocol_generated.h>

#include "events.hpp"

void get_ctrl_as_netprotocol(flatbuffers::FlatBufferBuilder &builder, SharedData *shared_data);
#endif // __CTRL_H__
