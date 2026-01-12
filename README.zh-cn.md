# Relink

Resolve RTL USB network adapter downgraded 100M issue on Windows wake up.
解决 Windows 休眠唤醒后 RTL USB 网卡协商速率降级至 100Mbps 的问题。

[English README](README.md)

---

## 概述

在 Windows 11 (25H2?) 环境下，RTL8156B USB 网卡在系统从休眠状态唤醒后，连接速率可能会降级至 100Mbps。即使安装了 Realtek 官网最新的驱动程序，并在设备管理器中关闭了所有节能选项，甚至强制设置协商速率为 2.5G，该问题依然存在。

本项目旨在解决此问题。其工作原理是：当系统唤醒后，程序会检测网卡速率。如果检测到速率为 100Mbps，则自动断开并重连网卡，从而强制重新协商速率。

*注意：由于缺乏其他设备进行广泛测试，本方案仅在 Windows 11 25H2 环境搭配 RTL8156B 网卡上进行了验证。*

---

## 更新

经过进一步测试，发现开启 BIOS/UEFI 中的“安全启动 (Secure Boot)”功能后，该网卡降速问题得到了解决。

如果您不希望开启安全启动，或者开启后问题仍未解决，可以尝试使用 Relink 来解决休眠唤醒后的降速烦恼。