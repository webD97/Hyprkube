import { CellContext } from "@tanstack/react-table";
import dayjs from "dayjs";
import LocalizedFormat from "dayjs/plugin/localizedFormat";
import RelativeTimePlugin from "dayjs/plugin/relativeTime";
import { DisplayableResource, ResourceField } from "../../hooks/useResourceWatch";
import RelativeTime from "../RelativeTime";

dayjs.extend(RelativeTimePlugin);
dayjs.extend(LocalizedFormat);

export const CustomCell: React.FC<CellContext<[string, DisplayableResource], unknown>> = (props) => {
    return (props.getValue() as ResourceField)?.components?.map((inner, idx) => {
        if ("PlainString" in inner) {
            return <span key={idx}>{inner.PlainString}</span>;
        }
        else if ("ColoredString" in inner) {
            const { string, color } = inner.ColoredString;
            return <span key={idx} style={{ color }}>{string}</span>;
        }
        else if ("ColoredBox" in inner) {
            const { color } = inner.ColoredBox;
            return <span key={idx} style={{ color }}>■{"\u00A0"}</span>;
        }
        else if ("Hyperlink" in inner) {
            const { url, display_text } = inner.Hyperlink;
            return <a key={idx} style={{ cursor: "pointer" }} onClick={() => open(url)} title={url}>🔗&nbsp;{display_text}</a>;
        }
        else if ("RelativeTime" in inner) {
            const { iso } = inner.RelativeTime;
            const date = dayjs(iso);
            return <span key={idx} title={date.toDate().toLocaleString()}><RelativeTime timestamp={iso} /></span>;
        }

        return <span key={idx}>(Unhandled)</span>;
    });
}