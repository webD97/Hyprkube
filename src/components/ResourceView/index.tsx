import { ResourceViewData } from "../../hooks/useResourceWatch";
import EmojiHint from "../EmojiHint";
import { open } from '@tauri-apps/plugin-shell';

export interface ResourceViewProps {
    columnTitles: string[],
    resourceData: ResourceViewData,
    onResourceClicked?: (uid: string) => void,
}

const ResourceView: React.FC<ResourceViewProps> = (props) => {
    const {
        columnTitles,
        resourceData = {},
        onResourceClicked = () => undefined,
    } = props;

    return (
        <>
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
                                                console.log({ part })
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
                                            })
                                        }

                                        return (
                                            <td key={idx}>
                                                {render}
                                            </td>
                                        );
                                    })
                                }
                            </tr>
                        ))
                    }
                </tbody>
            </table>
            {
                Object.keys(resourceData).length == 0
                    ? <EmojiHint emoji="🤷‍♀️">No resource of this type found.</EmojiHint>
                    : null
            }
        </>
    );
}

export default ResourceView;
