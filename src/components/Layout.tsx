import { ReactNode } from "react";
import { Sidebar } from "./Sidebar.tsx";
import { Menu, Plus } from "lucide-react";

interface LayoutProps {
  children: ReactNode;
  activeTab: string;
  onTabChange: (tab: string) => void;
  onAddTask: () => void;
  onOpenSettings: () => void;
  onOpenHistory: () => void;
}

const TAB_NAMES: Record<string, string> = {
  all: "全部任务",
  downloading: "下载中",
  completed: "已完成",
};

export function Layout(
  {
    children,
    activeTab,
    onTabChange,
    onAddTask,
    onOpenSettings,
    onOpenHistory,
  }: LayoutProps,
) {
  return (
    <div className="drawer lg:drawer-open bg-linear-to-br from-base-100 via-base-100 to-base-200/30 h-screen font-sans">
      <input id="my-drawer-2" type="checkbox" className="drawer-toggle" />

      {/* Page Content */}
      <div className="drawer-content flex flex-col h-full overflow-hidden">
        {/* Navbar with enhanced styling */}
        <div
          className="navbar bg-base-100/70 backdrop-blur-xl sticky top-0 z-10 px-4 md:px-8 h-16 border-b border-base-300/30 shadow-sm"
          data-tauri-drag-region
        >
          <div className="flex-none lg:hidden">
            <label
              htmlFor="my-drawer-2"
              aria-label="open sidebar"
              className="btn btn-square btn-ghost hover:bg-base-200"
            >
              <Menu className="w-6 h-6" />
            </label>
          </div>
          <div className="flex-1">
            <h2 className="text-xl font-bold px-2 tracking-tight text-base-content flex items-center gap-2">
              {TAB_NAMES[activeTab]}
              {activeTab === "downloading" && (
                <span className="inline-flex h-2 w-2 rounded-full bg-primary animate-pulse">
                </span>
              )}
            </h2>
          </div>
          <div className="flex-none">
            <button
              type="button"
              onClick={onAddTask}
              className="btn btn-primary btn-sm md:btn-md gap-2 shadow-lg shadow-primary/25 hover:shadow-xl hover:shadow-primary/35 hover:scale-105 transition-all duration-200 group"
            >
              <Plus className="w-4 h-4 group-hover:rotate-90 transition-transform duration-200" />
              <span className="hidden sm:inline font-semibold">新建任务</span>
            </button>
          </div>
        </div>

        {/* Main Content Area with improved spacing */}
        <main className="flex-1 overflow-y-auto p-4 md:p-8 bg-linear-to-b from-transparent to-base-200/20">
          <div className="max-w-5xl mx-auto space-y-4 animate-in fade-in">
            {children}
          </div>
        </main>
      </div>

      {/* Sidebar */}
      <Sidebar
        activeTab={activeTab}
        onTabChange={onTabChange}
        onOpenSettings={onOpenSettings}
        onOpenHistory={onOpenHistory}
      />
    </div>
  );
}
