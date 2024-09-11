import "./App.css"
import {onEventShowMenu} from "tauri-plugin-context-menu";
import {invoke, window as win} from "@tauri-apps/api";
import {useEffect, useState} from "react";
import {enable, isEnabled} from "tauri-plugin-autostart-api";

declare global {
    interface Window {
        getIcons: any
    }
}


onEventShowMenu('contextmenu', async (_e) => {
    return {
        theme: 'dark',
        items: [
            {
                label: "Fixed tab",
                disabled: false,
                event: async () => {
                    await invoke("fix_window")
                },
                checked: false
            },
            {
                label: "Close tab",
                disabled: false,
                event: async () => {
                    await invoke("close_window")
                }
            }
        ]
    };
});



function App() {

    const [title, setTitle] = useState("");
    const [files, setFiles] = useState([['']])

    useEffect(() => {
        win.getCurrent().title().then((prom) => {
            setTitle(prom)
        })
        isEnabled().then(async (prom) => {
            if (prom) { await enable() }
        })
    }, [setTitle])

    return (
        <div className={'w-screen h-screen overflow-hidden p-1'}
             onMouseEnter={ async (_ev) => {
                 if (!await win.getCurrent().isDecorated()) {
                     await invoke("expand_window")
                 }
                 let strPngArr: any[] = await invoke("get_files")
                 let base64Arr = []
                 for (let iconInfo of strPngArr) {
                     base64Arr.push([`data:image/png;base64,${iconInfo[0]}`, iconInfo[1], iconInfo[2]])
                 }
                 setFiles(base64Arr)
             }}
             onMouseLeave={async () => {
                 if (!await win.getCurrent().isDecorated()) {
                     await invoke("roll_up_window")
                 }

             }}
        >
            <h1 className={'text-lg'} id={'tab-title'}
                onDoubleClick={async () => {
                    const title = document.getElementById('tab-title')
                    if (title != null && title.contentEditable != "true") {
                        title.contentEditable = "true"
                    }
                }}
                onKeyDown={async (ev) => {
                    if (ev.key == "Enter") {
                        const title = document.getElementById('tab-title')
                        if (title != null && title.contentEditable != "false") {
                            title.contentEditable = "false"
                            await win.getCurrent().setTitle(title.textContent ? title.textContent : "null")
                            await invoke("save_changes")
                        }

                    }
                }}
            >{title}</h1>
            <div className={"m-2 h-auto flex flex-wrap justify-center"}>
                {files.map((file, index) => (
                    <figure key={index} className={'m-2 flex flex-col items-center text-xs w-[40px]'}>
                        <p><img className={'w-[30px]'} src={file[0]} alt="" onDoubleClick={async () => {
                            await invoke("run_app", {"pathBuf": file[2]})
                        }}/></p>
                        <figcaption className={'text-center'}>{file[1]}</figcaption>
                    </figure>
                ))}
            </div>
        </div>
    )
}

export default App;
