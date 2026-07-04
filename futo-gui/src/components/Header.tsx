import { Plus } from "lucide-react";

interface Props {
  count: number;
  activeCount: number;
  onAdd: () => void;
}

export function Header({ count, activeCount, onAdd }: Props) {
  return (
    <div className="flex items-center justify-between mb-5 px-0">
      <div>
        <h1 className="text-lg font-semibold m-0">Env manager</h1>
        <p className="text-[13px] text-futou-text-secondary mt-0.5">
          {count} service terinstall, {activeCount} aktif
        </p>
      </div>
      <button
        onClick={onAdd}
        className="inline-flex items-center gap-1.5 leading-none whitespace-nowrap font-sans text-[13px] bg-futou-text text-white border border-futou-text rounded-lg px-3 py-1.5 cursor-pointer active:scale-[0.98]"
      >
        <Plus size={16} />
        Add
      </button>
    </div>
  );
}
