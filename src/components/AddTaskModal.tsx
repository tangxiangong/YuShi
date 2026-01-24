import { useEffect, useRef, useState } from "react";
import { Download, FolderOpen, Link2, Loader2 } from "lucide-react";
import { addTask, getConfig } from "../commands.ts";

interface AddTaskModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export function AddTaskModal({ isOpen, onClose }: AddTaskModalProps) {
  const [url, setUrl] = useState("");
  const [dest, setDest] = useState("");
  const [loading, setLoading] = useState(false);
  const dialogRef = useRef<HTMLDivElement>(null);

  // Reset form and load default path when opened
  useEffect(() => {
    if (isOpen) {
      setUrl("");
      // Load default download path from config
      getConfig()
        .then((config) => {
          setDest(config.default_download_path);
        })
        .catch(() => {
          setDest("/tmp"); // Fallback
        });
    }
  }, [isOpen]);

  if (!isOpen) return null;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!url || !dest) return;

    setLoading(true);
    try {
      await addTask(url, dest);
      onClose();
    } catch (err) {
      console.error(err);
      alert("添加任务失败: " + err);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-start justify-center pt-[100px] animate-in fade-in">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/30 backdrop-blur-md transition-opacity"
        onClick={onClose}
      />

      {/* Modal Content */}
      <div
        ref={dialogRef}
        className="relative w-[520px] max-w-[90vw] bg-base-100 rounded-2xl shadow-2xl border border-base-300/50 overflow-hidden animate-in slide-in-from-top-4 duration-200"
      >
        {/* Header with gradient */}
        <div className="relative px-6 py-5 border-b border-base-200 bg-linear-to-r from-primary/5 via-secondary/5 to-primary/5">
          <div className="absolute inset-0 bg-linear-to-r from-transparent via-primary/5 to-transparent">
          </div>
          <div className="relative flex items-center justify-center gap-2">
            <div className="p-1.5 bg-primary/10 rounded-lg">
              <Download className="w-4 h-4 text-primary" />
            </div>
            <h3 className="text-base font-bold text-base-content">
              新建下载任务
            </h3>
          </div>
        </div>

        <form onSubmit={handleSubmit} className="p-6 space-y-6">
          {/* URL Input */}
          <div className="form-control space-y-2">
            <label className="label pb-1">
              <span className="label-text font-semibold flex items-center gap-2">
                <Link2 className="w-4 h-4 text-primary" />
                下载链接
              </span>
            </label>
            <input
              type="url"
              placeholder="https://example.com/file.zip"
              className="input input-bordered w-full h-11 text-sm focus:input-primary transition-all shadow-sm hover:shadow-md"
              value={url}
              onChange={(e) => setUrl(e.target.value)}
              autoFocus
              required
            />
            <label className="label pt-1">
              <span className="label-text-alt text-base-content/60">
                支持 HTTP、HTTPS 协议
              </span>
            </label>
          </div>

          {/* Path Input */}
          <div className="form-control space-y-2">
            <label className="label pb-1">
              <span className="label-text font-semibold flex items-center gap-2">
                <FolderOpen className="w-4 h-4 text-primary" />
                保存路径
              </span>
            </label>
            <div className="join w-full shadow-sm hover:shadow-md transition-all">
              <input
                type="text"
                placeholder="/Users/Downloads/..."
                className="input input-bordered join-item flex-1 h-11 text-sm font-mono focus:input-primary transition-all"
                value={dest}
                onChange={(e) => setDest(e.target.value)}
                required
              />
              <button
                type="button"
                className="btn join-item btn-outline h-11 min-h-0 font-medium hover:btn-primary transition-all"
              >
                <FolderOpen className="w-4 h-4" />
                浏览
              </button>
            </div>
            <label className="label pt-1">
              <span className="label-text-alt text-base-content/60">
                文件将保存到此目录
              </span>
            </label>
          </div>

          {/* Action Buttons */}
          <div className="pt-4 flex items-center justify-end gap-3 border-t border-base-200">
            <button
              type="button"
              onClick={onClose}
              disabled={loading}
              className="btn btn-ghost h-10 min-h-0 font-medium hover:bg-base-200 transition-all"
            >
              取消
            </button>
            <button
              type="submit"
              disabled={loading}
              className="btn btn-primary h-10 min-h-0 font-semibold text-primary-content shadow-lg shadow-primary/25 hover:shadow-xl hover:shadow-primary/35 hover:scale-105 transition-all flex items-center gap-2 group"
            >
              {loading
                ? (
                  <>
                    <Loader2 className="w-4 h-4 animate-spin" />
                    <span>添加中...</span>
                  </>
                )
                : (
                  <>
                    <Download className="w-4 h-4 group-hover:animate-bounce" />
                    <span>开始下载</span>
                  </>
                )}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
