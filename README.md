# Relink

> Resolve RTL USB network adapter downgraded 100M issue on Windows wake up.

---

[**中文(zh-cn) README**](README.zh-cn.md)

---

## Overview

On Windows 11 (specifically tested on 25H2), the RTL8156B USB network adapter may downgrade its negotiated link speed to 100Mbps after the system wakes from sleep. This issue persists even after installing the latest drivers from Realtek, disabling all power-saving options in Device Manager, and forcing the Speed & Duplex setting to 2.5Gbps Full Duplex.

**Relink** addresses this by detecting the network adapter's link speed upon system wake-up. If the speed is detected as 100Mbps, the tool automatically disconnects and reconnects the adapter, forcing a link speed renegotiation.

*Note: Testing has been limited to a Windows 11 25H2 environment with an RTL8156B adapter.*

---

## Update

Further testing revealed that enabling **Secure Boot** in the BIOS/UEFI resolved the issue for my specific case.

However, if you prefer not to enable Secure Boot, or if that solution does not work for you, Relink offers a software-based workaround to automatically restore your network speed after sleep.
