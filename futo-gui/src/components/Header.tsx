import { Plus, Power } from "lucide-react";

interface Props {
  count: number;
  activeCount: number;
  daemonRunning: boolean;
  onAdd: () => void;
  onToggleDaemon: () => void;
}

export function Header({ count, activeCount, daemonRunning, onAdd, onToggleDaemon }: Props) {
  return (
    <div className="flex items-center justify-between mb-5 px-0">
      <div>
        <h1 className="text-lg font-semibold m-0">Futou</h1>
        <p className="text-[13px] text-futou-text-secondary mt-0.5">
          {count} service, {activeCount} aktif &middot; Daemon{" "}
          <span className={daemonRunning ? "text-futou-success" : "text-futou-danger"}>
            {daemonRunning ? "ON" : "OFF"}
          </span>
        </p>
      </div>
      <div className="flex items-center gap-2">
        <button
          onClick={onToggleDaemon}
          title={daemonRunning ? "Stop daemon" : "Start daemon"}
          className={`inline-flex items-center gap-1 leading-none whitespace-nowrap font-sans text-[13px] border rounded-lg px-2.5 py-1.5 cursor-pointer active:scale-[0.98] ${daemonRunning ? "bg-futou-success-bg border-futou-success text-futou-success hover:bg-futou-success hover:text-white" : "bg-futou-danger-bg border-futou-danger text-futou-danger hover:bg-futou-danger hover:text-white"}`}
        >
          <Power size={14} />
        </button>
        <button
          onClick={onAdd}
          className="inline-flex items-center gap-1.5 leading-none whitespace-nowrap font-sans text-[13px] bg-futou-text text-white border border-futou-text rounded-lg px-3 py-1.5 cursor-pointer active:scale-[0.98]"
        >
          <Plus size={16} />
          Add
        </button>
      </div>
    </div>
  );
}
