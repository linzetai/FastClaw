import { useState, useEffect } from "react";
import { Upload, Settings, ArrowRight } from "lucide-react";
import * as transport from "../../lib/transport";

interface ImportChoiceStepProps {
  onSelect: (choice: "new" | "import") => void;
}

export function ImportChoiceStep({ onSelect }: ImportChoiceStepProps) {
  const [importStatus, setImportStatus] = useState<"idle" | "loading" | "success" | "error">("idle");
  const [errorMessage, setErrorMessage] = useState("");

  const handleImportClick = async () => {
    setImportStatus("loading");
    setErrorMessage("");
    
    try {
      if (transport.isTauri) {
        // 使用 Tauri dialog 打开文件选择器
        const selected = await window.__TAURI__.dialog.open({
          filters: [{
            name: "FastClaw Migration Files",
            extensions: ["json", "fcdata"]
          }],
          multiple: false
        });
        
        if (selected) {
          // 读取文件内容
          const fileContents = await window.__TAURI__.fs.readBinaryFile(selected);
          
          // 使用导入功能
          await transport.importData(new Uint8Array(fileContents), {
            merge: false,
            overwriteConfig: true,
            overwriteAgents: true,
            overwriteSessions: true,
            overwriteSkills: true
          });
          
          setImportStatus("success");
          setTimeout(() => onSelect("new"), 1500); // 导入成功后进入下一步
        } else {
          setImportStatus("idle");
        }
      } else {
        setImportStatus("error");
        setErrorMessage("迁移功能仅在桌面应用中可用");
      }
    } catch (error) {
      setImportStatus("error");
      setErrorMessage(error instanceof Error ? error.message : "导入失败");
      setTimeout(() => setImportStatus("idle"), 3000);
    }
  };

  return (
    <div className="flex flex-col items-center text-center">
      <h1
        className="mt-6 text-[28px] font-bold tracking-tight"
        style={{ color: "var(--fill-primary)" }}
      >
        导入现有配置
      </h1>
      <p
        className="mt-3 max-w-[380px] text-[15px] leading-relaxed"
        style={{ color: "var(--fill-secondary)" }}
      >
        从备份文件导入您的配置、代理、技能和会话数据
      </p>
      
      <div className="mt-8 w-full max-w-[320px]">
        <div
          className="rounded-[var(--radius-md)] p-6 cursor-pointer transition-all hover:scale-[1.02]"
          style={{ 
            background: "var(--bg-elevated)", 
            border: "2px dashed var(--separator-opaque)",
            minHeight: "160px",
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            justifyContent: "center"
          }}
          onClick={handleImportStatus !== "loading" ? handleImportClick : undefined}
        >
          {importStatus === "idle" && (
            <>
              <Upload size={48} style={{ color: "var(--fill-tertiary)" }} />
              <p className="mt-4 text-[14px]" style={{ color: "var(--fill-secondary)" }}>
                点击选择迁移文件
              </p>
              <p className="mt-1 text-[12px]" style={{ color: "var(--fill-tertiary)" }}>
                支持 .json 或 .fcdata 文件
              </p>
            </>
          )}
          
          {importStatus === "loading" && (
            <>
              <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-[var(--fill-primary)]"></div>
              <p className="mt-4 text-[14px]" style={{ color: "var(--fill-secondary)" }}>
                正在导入...
              </p>
            </>
          )}
          
          {importStatus === "success" && (
            <>
              <div style={{ color: "var(--green)" }}>✓</div>
              <p className="mt-4 text-[14px]" style={{ color: "var(--green)" }}>
                导入成功！
              </p>
            </>
          )}
          
          {importStatus === "error" && (
            <>
              <div style={{ color: "var(--red)" }}>✕</div>
              <p className="mt-4 text-[14px]" style={{ color: "var(--red)" }}>
                {errorMessage || "导入失败"}
              </p>
            </>
          )}
        </div>
        
        <div className="mt-6 flex gap-3">
          <button
            onClick={() => onSelect("new")}
            className="flex-1 flex cursor-pointer items-center justify-center gap-2 rounded-full px-4 py-2.5 text-[14px] font-medium transition-all duration-200 hover:scale-[1.02] active:scale-[0.98]"
            style={{
              background: "var(--bg-elevated)",
              color: "var(--fill-primary)",
              border: "1px solid var(--separator-opaque)"
            }}
          >
            <Settings size={14} strokeWidth={2} />
            手动配置
          </button>
        </div>
      </div>
    </div>
  );
}