#include <vita2d.h>

#include "events.hpp"

#ifndef DRAW_HELPER_HPP
#define DRAW_HELPER_HPP

extern vita2d_pgf *debug_font;
extern uint32_t need_color;
extern uint32_t common_color;
extern uint32_t error_color;
extern uint32_t done_color;

void draw_pad_mode(uint32_t *events, bool *connected_to_network, bool *pc_connect_state,
                   char *vita_ip, SceNetCtlInfo *info, SharedData *shared_data);

#endif // DRAW_HELPER_HPP
