#ifndef BT_PAIRING_H
#define BT_PAIRING_H

#include <stdbool.h>
#include <stdint.h>

void bt_pairing_init(uint8_t button_gpio);
void bt_pairing_enter(void);
void bt_pairing_exit(void);
bool bt_pairing_is_active(void);

#endif
