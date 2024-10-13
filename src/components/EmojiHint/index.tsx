import React, { ReactNode } from "react";

import classes from './component.module.css';

export interface EmojiHintProps {
    children?: ReactNode,
    emoji: string,
}

const EmojiHint: React.FC<EmojiHintProps> = (props) => {
    const { emoji = '🤯', children } = props;

    return (
        <div className={classes.container}>
            <p>{emoji}</p>
            {children ? <p>{children}</p> : null}
        </div>
    );
};

export default EmojiHint;
