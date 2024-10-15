import { ResourceViewData } from "../../hooks/useResourceWatch";
import EmojiHint from "../EmojiHint";

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
                                    columnData.map((data, idx) => {
                                        let render = null;

                                        if ("Err" in data) {
                                            render = data.Err.message;
                                        }

                                        if ("Ok" in data) {
                                            if ("PlainString" in data.Ok) {
                                                render = data.Ok.PlainString;
                                            }
                                            else if ("ColoredString" in data.Ok) {
                                                const [value, color] = data.Ok.ColoredString;
                                                render = <span style={{ color }}>{value}</span>;
                                            }
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
