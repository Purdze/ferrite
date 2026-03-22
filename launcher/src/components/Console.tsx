import { invoke } from "@tauri-apps/api/core";
import "../styles.css";
import Titlebar from "./Titlebar";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState, useRef } from "react";
import { HiClipboardCopy } from "react-icons/hi";

const getLogs: () => Promise<string[]> = async () => invoke("get_client_logs");

const Log = ({ log }: { log: string }) => {
  let split = log.split("]");
  let start_str = split[0];
  let message = split[1];

  let type = "INFO";

  for (const tag of ["WARN", "INFO", "DEBUG"]) {
    if (start_str.includes(tag)) {
      type = tag;
    }
  }

  return (
    <p className="console-log">
      <span className={`console-text-${type.toLowerCase()}`}>{start_str}]</span>
      {message}
    </p>
  );
};

export default function Console() {
  const [logs, setLogs] = useState<string[]>([]);
  const bottomRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    const do_shit = async () => {
      setLogs(await getLogs());

      listen<string>("console_message", (event) => {
        console.log(event.payload);
        setLogs((prev) => [...prev, event.payload]);
      });
    };

    do_shit();
  }, []);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [logs]);

  const copyLogs = () => {
    let copyText = logs.join("\n");
    navigator.clipboard.writeText(copyText);
  };

  return (
    <div className="app">
      <Titlebar name="POMC Debugger" />
      <div className="console-holder">
        <div className="console">
          <div className="console-scroll">
            {logs.map((object, key) => (
              <Log log={object} key={key} />
            ))}
            <div ref={bottomRef} /> {/* element at the bottom that can be scrolled to */}
          </div>
        </div>
        <div className="console-bottom-bar">
          <button className="console-button" onClick={copyLogs}>
            <HiClipboardCopy className="clipboard-icon" />
          </button>
        </div>
      </div>
    </div>
  );
}
