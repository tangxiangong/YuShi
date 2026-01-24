import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { Layout } from "./components/Layout.tsx";
import { TaskItem } from "./components/TaskItem.tsx";
import { AddTaskModal } from "./components/AddTaskModal.tsx";
import { SettingsModal } from "./components/SettingsModal.tsx";
import { HistoryModal } from "./components/HistoryModal.tsx";
import { UpdateModal } from "./components/UpdateModal.tsx";
import { DownloadTask, QueueEvent } from "./types.ts";
import { getConfig, getTasks } from "./commands.ts";
import { Inbox, Plus } from "lucide-react";

function App() {
  const [tasks, setTasks] = useState<DownloadTask[]>([]);
  const [activeTab, setActiveTab] = useState("all");
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [isHistoryOpen, setIsHistoryOpen] = useState(false);
  const [isUpdateOpen, setIsUpdateOpen] = useState(false);

  const fetchTasks = () => {
    getTasks().then(setTasks).catch(console.error);
  };

  useEffect(() => {
    fetchTasks();

    // Load theme from backend config
    getConfig().then((config) => {
      const root = document.documentElement;
      const theme = config.theme;

      if (theme === "system") {
        const systemTheme =
          globalThis.matchMedia("(prefers-color-scheme: dark)").matches
            ? "dim"
            : "cupcake";
        root.setAttribute("data-theme", systemTheme);
      } else {
        root.setAttribute("data-theme", theme === "dark" ? "dim" : "cupcake");
      }
    }).catch((err) => {
      console.error("Failed to load theme:", err);
      // Fallback to system theme
      const root = document.documentElement;
      const systemTheme =
        globalThis.matchMedia("(prefers-color-scheme: dark)").matches
          ? "dim"
          : "cupcake";
      root.setAttribute("data-theme", systemTheme);
    });

    // Listen for events
    const unlisten = listen<QueueEvent>("download-event", (event) => {
      const data = event.payload;

      if (data.type === "TaskAdded") {
        fetchTasks();
        return;
      }

      setTasks((prevTasks) => {
        if (!data.payload || !("task_id" in data.payload)) return prevTasks;

        const taskId = data.payload.task_id;
        const taskIndex = prevTasks.findIndex((t) => t.id === taskId);

        if (taskIndex === -1) {
          fetchTasks();
          return prevTasks;
        }

        const newTasks = [...prevTasks];
        const task = { ...newTasks[taskIndex] };

        switch (data.type) {
          case "TaskProgress":
            task.downloaded = data.payload.downloaded;
            task.total_size = data.payload.total;
            task.speed = data.payload.speed;
            task.eta = data.payload.eta;
            task.status = "Downloading";
            break;
          case "TaskStarted":
            task.status = "Downloading";
            break;
          case "TaskPaused":
            task.status = "Paused";
            break;
          case "TaskResumed":
            task.status = "Pending";
            break;
          case "TaskCompleted":
            task.status = "Completed";
            task.speed = 0;
            task.eta = undefined;
            break;
          case "TaskFailed":
            task.status = "Failed";
            task.error = data.payload.error;
            task.speed = 0;
            break;
          case "TaskCancelled":
            task.status = "Cancelled";
            break;
        }

        newTasks[taskIndex] = task;
        return newTasks;
      });
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  // Filter tasks
  const filteredTasks = tasks.filter((task) => {
    if (activeTab === "downloading") {
      return task.status === "Downloading" || task.status === "Pending";
    }
    if (activeTab === "completed") {
      return task.status === "Completed";
    }
    return true;
  });

  // Sort: Downloading first, then by created_at desc
  filteredTasks.sort((a, b) => {
    const aActive = a.status === "Downloading" || a.status === "Pending";
    const bActive = b.status === "Downloading" || b.status === "Pending";

    if (aActive && !bActive) return -1;
    if (!aActive && bActive) return 1;

    return b.created_at - a.created_at;
  });

  return (
    <Layout
      activeTab={activeTab}
      onTabChange={setActiveTab}
      onAddTask={() => setIsModalOpen(true)}
      onOpenSettings={() => setIsSettingsOpen(true)}
      onOpenHistory={() => setIsHistoryOpen(true)}
    >
      {filteredTasks.length === 0
        ? (
          <div className="flex flex-col items-center justify-center py-24 px-4 animate-in fade-in">
            <div className="relative mb-8">
              <div className="absolute inset-0 bg-linear-to-r from-primary/20 via-secondary/20 to-primary/20 blur-3xl opacity-50 animate-pulse">
              </div>
              <div className="relative bg-linear-to-br from-base-200 to-base-300 p-6 rounded-3xl shadow-xl">
                <Inbox className="w-20 h-20 text-base-content/30" />
              </div>
            </div>
            <h3 className="text-xl font-bold text-base-content mb-2">
              暂无下载任务
            </h3>
            <p className="text-sm text-base-content/60 mb-6 text-center max-w-sm">
              {activeTab === "downloading" && "当前没有正在下载的任务"}
              {activeTab === "completed" && "还没有完成的下载任务"}
              {activeTab === "all" && "开始你的第一个下载任务吧"}
            </p>
            <button
              type="button"
              onClick={() => setIsModalOpen(true)}
              className="btn btn-primary gap-2 shadow-lg shadow-primary/25 hover:shadow-xl hover:shadow-primary/35 hover:scale-105 transition-all group"
            >
              <Plus className="w-5 h-5 group-hover:rotate-90 transition-transform duration-200" />
              <span className="font-semibold">新建下载任务</span>
            </button>
          </div>
        )
        : (
          <div className="space-y-4">
            {filteredTasks.map((task) => (
              <TaskItem
                key={task.id}
                task={task}
                onRefreshNeeded={fetchTasks}
              />
            ))}
          </div>
        )}
      <AddTaskModal
        isOpen={isModalOpen}
        onClose={() => {
          setIsModalOpen(false);
          fetchTasks();
        }}
      />
      <SettingsModal
        isOpen={isSettingsOpen}
        onClose={() => setIsSettingsOpen(false)}
        onOpenUpdate={() => setIsUpdateOpen(true)}
      />
      <HistoryModal
        isOpen={isHistoryOpen}
        onClose={() => setIsHistoryOpen(false)}
      />
      <UpdateModal
        isOpen={isUpdateOpen}
        onClose={() => setIsUpdateOpen(false)}
      />
    </Layout>
  );
}

export default App;
