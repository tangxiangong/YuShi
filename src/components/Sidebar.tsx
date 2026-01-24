import { cn } from "../lib/utils.ts";
import { CheckCircle, Download, History, List, Settings } from "lucide-react";

interface SidebarProps {
  activeTab: string;
  onTabChange: (tab: string) => void;
  onOpenSettings: () => void;
  onOpenHistory: () => void;
}

export function Sidebar(
  { activeTab, onTabChange, onOpenSettings, onOpenHistory }: SidebarProps,
) {
  const tabs = [
    { id: "all", label: "全部任务", icon: List },
    { id: "downloading", label: "下载中", icon: Download },
    { id: "completed", label: "已完成", icon: CheckCircle },
  ];

  return (
    <div className="drawer-side z-20">
      <label
        htmlFor="my-drawer-2"
        aria-label="close sidebar"
        className="drawer-overlay"
      >
      </label>
      <div className="bg-linear-to-b from-base-200 to-base-100 text-base-content min-h-full w-80 p-4 flex flex-col pt-10 shadow-xl border-r border-base-300/50">
        {/* Draggable Titlebar Region */}
        <div
          data-tauri-drag-region
          className="absolute top-0 left-0 w-full h-10 z-50"
        />

        {/* Header with gradient accent */}
        <div className="mb-8 px-4 flex items-center gap-3">
          <div className="relative">
            <div className="absolute inset-0 bg-linear-to-br from-primary to-secondary opacity-20 blur-xl rounded-full">
            </div>
            <div className="relative bg-linear-to-br from-primary to-secondary p-2.5 rounded-xl shadow-lg">
              <Download className="w-6 h-6 text-primary-content" />
            </div>
          </div>
          <div className="flex flex-col">
            <span className="text-xl font-bold tracking-tight bg-linear-to-r from-primary to-secondary bg-clip-text text-transparent">
              YuShi
            </span>
            <span className="text-xs text-base-content/50 font-medium">
              下载管理器
            </span>
          </div>
        </div>

        {/* Navigation with improved styling */}
        <ul className="menu menu-lg rounded-box w-full gap-2 flex-1">
          {tabs.map((tab) => (
            <li key={tab.id}>
              <a
                onClick={() => onTabChange(tab.id)}
                className={cn(
                  "group relative overflow-hidden rounded-xl transition-all duration-200",
                  activeTab === tab.id
                    ? "active font-semibold bg-primary text-primary-content shadow-lg shadow-primary/20"
                    : "text-base-content/70 hover:text-base-content hover:bg-base-200/50",
                )}
              >
                <tab.icon
                  className={cn(
                    "w-5 h-5 transition-transform duration-200",
                    activeTab === tab.id
                      ? "scale-110"
                      : "group-hover:scale-105",
                  )}
                />
                <span className="relative z-10">{tab.label}</span>
                {activeTab === tab.id && (
                  <div className="absolute inset-0 bg-linear-to-r from-primary/0 via-primary-content/10 to-primary/0 animate-pulse">
                  </div>
                )}
              </a>
            </li>
          ))}
        </ul>

        {/* Decorative divider */}
        <div className="divider my-2 opacity-50"></div>

        {/* Footer Settings with enhanced styling */}
        <div className="pt-2">
          <ul className="menu menu-lg w-full gap-1">
            <li>
              <a
                onClick={onOpenHistory}
                className="group text-base-content/70 hover:text-base-content hover:bg-base-200/50 rounded-xl transition-all duration-200"
              >
                <History className="w-5 h-5 group-hover:scale-110 transition-transform duration-200" />
                <span>历史记录</span>
              </a>
            </li>
            <li>
              <a
                onClick={onOpenSettings}
                className="group text-base-content/70 hover:text-base-content hover:bg-base-200/50 rounded-xl transition-all duration-200"
              >
                <Settings className="w-5 h-5 group-hover:rotate-90 transition-transform duration-300" />
                <span>设置</span>
              </a>
            </li>
          </ul>
        </div>

        {/* Version info */}
        <div className="mt-4 px-4 py-2 text-center">
          <p className="text-xs text-base-content/40 font-medium">v0.1.0</p>
        </div>
      </div>
    </div>
  );
}
