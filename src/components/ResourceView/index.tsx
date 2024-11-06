import dayjs from "dayjs";
import RelativeTime from "dayjs/plugin/relativeTime";
import LocalizedFormat from "dayjs/plugin/localizedFormat";
import { ResourceViewData } from "../../hooks/useResourceWatch";
import EmojiHint from "../EmojiHint";
import { open } from '@tauri-apps/plugin-shell';

import styles from './styles.module.css';

export interface ResourceViewProps {
    namespace?: string,
    resourceNamePlural?: string,
    columnTitles: string[],
    resourceData: ResourceViewData,
    onResourceClicked?: (uid: string) => void,
    onDeleteClicked?: (uid: string) => void,
}

dayjs.extend(RelativeTime);
dayjs.extend(LocalizedFormat);

const ResourceView: React.FC<ResourceViewProps> = (props) => {
    const {
        namespace,
        resourceNamePlural,
        columnTitles,
        resourceData = {},
        onResourceClicked = () => undefined,
        onDeleteClicked = () => undefined,
    } = props;

    return (
        <div className={styles.container}>
            <table>
                <thead>
                    <tr>
                        {
                            columnTitles.map((title, idx) => (
                                <th key={`${title}@${idx}`}>
                                    {title}
                                </th>
                            ))
                        }
                        <th>Actions</th>
                    </tr>
                </thead>
                <tbody>
                    {
                        Object.entries(resourceData).map(([uid, columnData]) => (
                            <tr key={uid} onClick={() => onResourceClicked(uid)}>
                                {
                                    columnData.columns.map((data, idx) => {
                                        let render = null;

                                        if ("Err" in data) {
                                            render = data.Err;
                                        }

                                        if ("Ok" in data) {
                                            render = data.Ok.map((part, idx) => {
                                                if ("PlainString" in part) {
                                                    return <span key={idx}>{part.PlainString}</span>;
                                                }
                                                else if ("ColoredString" in part) {
                                                    const { string, color } = part.ColoredString;
                                                    return <span key={idx} style={{ color }}>{string}</span>;
                                                }
                                                else if ("ColoredBox" in part) {
                                                    const { color } = part.ColoredBox;
                                                    return <span key={idx} style={{ color }}>■{"\u00A0"}</span>;
                                                }
                                                else if ("Hyperlink" in part) {
                                                    const { url, display_text } = part.Hyperlink;
                                                    return <a key={idx} style={{ cursor: "pointer" }} onClick={() => open(url)} title={url}>🔗&nbsp;{display_text}</a>;
                                                }
                                                else if ("RelativeTime" in part) {
                                                    const { iso } = part.RelativeTime;
                                                    const date = dayjs(iso);
                                                    return <span key={idx} title={date.format("LLL")}>{dayjs().to(date, true)}</span>;
                                                }
                                            })
                                        }

                                        return (
                                            <td key={idx}>
                                                {render}
                                            </td>
                                        );
                                    })
                                }
                                <td className={styles.resourceQuickActions}>
                                    <button onClick={(e) => {
                                        e.stopPropagation();
                                        onDeleteClicked(columnData.uid);
                                    }}>Delete</button>
                                </td>
                            </tr>
                        ))
                    }
                </tbody>
            </table >
            {
                Object.keys(resourceData).length == 0
                    ? <EmojiHint emoji="⏳">No {resourceNamePlural} in namespace "{namespace}" yet</EmojiHint>
                    : null
            }
        </div>
    );
}

export default ResourceView;
