import { useEffect, useState } from "react";
import {
  AlertCircle,
  CheckCircle,
  Download,
  Loader2,
  RefreshCw,
  Sparkles,
  X,
} from "lucide-react";
import { checkForUpdates, downloadAndInstallUpdate } from "../commands.ts";
import type { UpdateInfo } from "../types.ts";
import { listen } from "@tauri-apps/api/event";

interface UpdateModalProps {
  isOpen: boolean;
  onClose: () => void;
}

type UpdateState =
  | "checking"
  | "available"
  | "no-update"
  | "downloading"
  | "installing"
  | "error";

export function UpdateModal({ isOpen, onClose }: UpdateModalProps) {
  const [state, setState] = useState<UpdateState>("checking");
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [downloadProgress, setDownloadProgress] = useState(0);
  const [error, setError] = useState<string>("");

  useEffect(() => {
    if (isOpen) {
      checkUpdate();

      // 监听更新事件
      const unlistenProgress = listen<number>(
        "update-download-progress",
        (event) => {
          setDownloadProgress(event.payload);
        },
      );

      const unlistenFinished = listen("update-download-finished", () => {
        setState("installing");
      });

      const unlistenInstalling = listen("update-installing", () => {
        setState("installing");
      });

      return () => {
        unlistenProgress.then((f) => f());
        unlistenFinished.then((f) => f());
        unlistenInstalling.then((f) => f());
      };
    }
  }, [isOpen]);

  const checkUpdate = async () => {
    setState("checking");
    setError("");

    try {
      const info = await checkForUpdates();
      setUpdateInfo(info);

      if (info.available) {
        setState("available");
      } else {
        setState("no-update");
      }
    } catch (err) {
      setError(String(err));
      setState("error");
    }
  };

  const handleUpdate = async () => {
    setState("downloading");
    setDownloadProgress(0);

    try {
      await downloadAndInstallUpdate();
      // 安装后应用会自动重启
    } catch (err) {
      setError(String(err));
      setState("error");
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center animate-in fade-in">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/30 backdrop-blur-md"
        onClick={state !== "downloading" && state !== "installing"
          ? onClose
          : undefined}
      />

      {/* Modal Content */}
      <div className="relative w-[500px] max-w-[90vw] bg-base-100 rounded-2xl shadow-2xl overflow-hidden animate-in zoom-in-95 duration-200 border border-base-300/50">
        {/* Header */}
        <div className="px-6 py-5 border-b border-base-200 bg-linear-to-r from-primary/5 via-secondary/5 to-primary/5 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-primary/10 rounded-lg">
              <Sparkles className="w-5 h-5 text-primary" />
            </div>
            <h2 className="text-lg font-bold text-base-content">检查更新</h2>
          </div>
          {state !== "downloading" && state !== "installing" && (
            <button
              type="button"
              onClick={onClose}
              className="btn btn-sm btn-ghost btn-square"
            >
              <X className="w-5 h-5" />
            </button>
          )}
        </div>

        {/* Content */}
        <div className="p-8">
          {/* Checking State */}
          {state === "checking" && (
            <div className="flex flex-col items-center gap-4 py-8">
              <Loader2 className="w-12 h-12 text-primary animate-spin" />
              <p className="text-base-content/70">正在检查更新...</p>
            </div>
          )}

          {/* No Update State */}
          {state === "no-update" && updateInfo && (
            <div className="flex flex-col items-center gap-4 py-8">
              <div className="p-4 bg-success/10 rounded-full">
                <CheckCircle className="w-12 h-12 text-success" />
              </div>
              <div className="text-center">
                <h3 className="text-lg font-semibold text-base-content mb-2">
                  已是最新版本
                </h3>
                <p className="text-sm text-base-content/60">
                  当前版本: v{updateInfo.current_version}
                </p>
              </div>
            </div>
          )}

          {/* Update Available State */}
          {state === "available" && updateInfo && (
            <div className="space-y-6">
              <div className="flex items-center gap-4 p-4 bg-primary/5 rounded-xl border border-primary/20">
                <div className="p-3 bg-primary/10 rounded-lg">
                  <Download className="w-8 h-8 text-primary" />
                </div>
                <div className="flex-1">
                  <h3 className="text-lg font-semibold text-base-content">
                    发现新版本
                  </h3>
                  <p className="text-sm text-base-content/60">
                    v{updateInfo.current_version} → v{updateInfo.latest_version}
                  </p>
                </div>
              </div>

              {updateInfo.body && (
                <div className="space-y-2">
                  <h4 className="text-sm font-semibold text-base-content">
                    更新内容：
                  </h4>
                  <div className="p-4 bg-base-200 rounded-lg max-h-48 overflow-y-auto">
                    <pre className="text-xs text-base-content/70 whitespace-pre-wrap font-sans">
                      {updateInfo.body}
                    </pre>
                  </div>
                </div>
              )}

              {updateInfo.date && (
                <p className="text-xs text-base-content/50">
                  发布时间: {new Date(updateInfo.date).toLocaleString("zh-CN")}
                </p>
              )}
            </div>
          )}

          {/* Downloading State */}
          {state === "downloading" && (
            <div className="space-y-6 py-4">
              <div className="flex flex-col items-center gap-4">
                <Loader2 className="w-12 h-12 text-primary animate-spin" />
                <div className="text-center">
                  <h3 className="text-lg font-semibold text-base-content mb-2">
                    正在下载更新
                  </h3>
                  <p className="text-sm text-base-content/60">
                    请稍候，不要关闭应用...
                  </p>
                </div>
              </div>

              <div className="space-y-2">
                <div className="flex justify-between text-sm">
                  <span className="text-base-content/70">下载进度</span>
                  <span className="font-semibold text-primary">
                    {downloadProgress}%
                  </span>
                </div>
                <progress
                  className="progress progress-primary w-full"
                  value={downloadProgress}
                  max="100"
                >
                </progress>
              </div>
            </div>
          )}

          {/* Installing State */}
          {state === "installing" && (
            <div className="flex flex-col items-center gap-4 py-8">
              <Loader2 className="w-12 h-12 text-success animate-spin" />
              <div className="text-center">
                <h3 className="text-lg font-semibold text-base-content mb-2">
                  正在安装更新
                </h3>
                <p className="text-sm text-base-content/60">
                  应用将自动重启...
                </p>
              </div>
            </div>
          )}

          {/* Error State */}
          {state === "error" && (
            <div className="space-y-6">
              <div className="flex flex-col items-center gap-4 py-4">
                <div className="p-4 bg-error/10 rounded-full">
                  <AlertCircle className="w-12 h-12 text-error" />
                </div>
                <div className="text-center">
                  <h3 className="text-lg font-semibold text-base-content mb-2">
                    更新失败
                  </h3>
                  <p className="text-sm text-error">{error}</p>
                </div>
              </div>
            </div>
          )}
        </div>

        {/* Footer Actions */}
        {(state === "available" || state === "error" ||
          state === "no-update") && (
          <div className="px-8 py-4 border-t border-base-200 bg-base-200/30 flex justify-end gap-3">
            {state === "error" && (
              <button
                type="button"
                onClick={checkUpdate}
                className="btn btn-ghost h-10 min-h-0 font-medium"
              >
                <RefreshCw className="w-4 h-4" />
                重试
              </button>
            )}
            {state !== "available" && (
              <button
                type="button"
                onClick={onClose}
                className="btn btn-ghost h-10 min-h-0 font-medium"
              >
                关闭
              </button>
            )}
            {state === "available" && (
              <>
                <button
                  type="button"
                  onClick={onClose}
                  className="btn btn-ghost h-10 min-h-0 font-medium"
                >
                  稍后提醒
                </button>
                <button
                  type="button"
                  onClick={handleUpdate}
                  className="btn btn-primary h-10 min-h-0 font-semibold text-primary-content shadow-lg shadow-primary/25 hover:shadow-xl hover:shadow-primary/35 hover:scale-105 transition-all flex items-center gap-2"
                >
                  <Download className="w-4 h-4" />
                  立即更新
                </button>
              </>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
