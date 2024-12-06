#include "draw_helper.hpp"
#include "status.hpp"

#include <common.h>

vita2d_pgf *debug_font;
uint32_t need_color = 0;
uint32_t common_color = RGBA8(0xFF, 0xFF, 0xFF, 0xFF); // White color
uint32_t error_color = RGBA8(0xFF, 0x00, 0x00, 0xFF);  // Bright red color
uint32_t done_color = RGBA8(0x00, 0xFF, 0x00, 0xFF);   // Bright green color

void draw_pad_mode(uint32_t *events, bool *connected_to_network, bool *pc_connect_state, char *vita_ip,
                   SceNetCtlInfo *info, SharedData *shared_data) {
  vita2d_pgf_draw_text(debug_font, 2, 20, common_color, 1.0,
                       "VitaOxiPad v1.2.0 \nbuild " __DATE__ ", " __TIME__);

  if (*events & MainEvent::NET_CONNECT) {
    *connected_to_network = true;
    sceNetCtlInetGetInfo(SCE_NETCTL_INFO_GET_IP_ADDRESS, info);
    snprintf(vita_ip, INET_ADDRSTRLEN, "%s", info->ip_address);
  } else if (*events & MainEvent::NET_DISCONNECT) {
    *connected_to_network = false;
  }

  if (*connected_to_network) {
    vita2d_pgf_draw_textf(debug_font, 750, 20, common_color, 1.0, "Listening on:\nIP: %s\nPort: %d",
                          vita_ip, NET_PORT);
  } else {
    vita2d_pgf_draw_text(debug_font, 750, 20, error_color, 1.0, "Not connected\nto a network :(");
  }

  if (*events & MainEvent::PC_CONNECT) {
    *pc_connect_state = true;
  } else if (*events & MainEvent::PC_DISCONNECT) {
    *pc_connect_state = false;
  }
  if (*pc_connect_state) {
    vita2d_pgf_draw_textf(debug_font, 2, 540, done_color, 1.0, "Status: Connected (%s)",
                          shared_data->client_ip);
  } else {
    vita2d_pgf_draw_text(debug_font, 2, 540, error_color, 1.0, "Status: Not connected :(");
  }

  if (shared_data->charger_connected) {
    need_color = done_color;
  } else if (shared_data->battery_level < 30) {
    need_color = error_color;
  } else {
    need_color = common_color;
  }
  vita2d_pgf_draw_textf(debug_font, 785, 520, need_color, 1.0, "Battery: %s%d%%",
                        shared_data->charger_connected ? "+" : "", shared_data->battery_level);

  if (shared_data->wifi_signal_strength < 50) {
    need_color = error_color;
  } else {
    need_color = common_color;
  }
  vita2d_pgf_draw_textf(debug_font, 785, 540, need_color, 1.0, "WiFi signal: %d%%",
                        shared_data->wifi_signal_strength);
}
