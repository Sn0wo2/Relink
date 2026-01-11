# Relink

Resolve the "USB network adapter downgrading from 2.5G to 100M" negotiation issue on Windows wake-up
解决 Windows 唤醒后 USB 网卡从 2.5G 降级到 100M 的协商问题。

---

## Overview
在 Windows 下我的 RTL8156B USB网卡休眠后唤醒会协商掉速到100Mbps，即使装了官网最新驱动、在设备管理器关掉了任何可能的节能选项并强制设置协商为2.5G也无法解决。
因为没有其他设备测试，所以我仅针对我的 Windows 11 环境和 RTL8156B 进行测试。

具体原理就是唤醒后检测到网卡速率是100Mbps就断开重连网卡，从而强制重新协商速率。
