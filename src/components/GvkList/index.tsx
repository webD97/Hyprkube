import React from 'react';
import { Gvk } from '../../model/k8s';

import classes from './component.module.css';

export interface GvkListProps {
    className?: string,
    gvks: Gvk[],
    withGroupName?: boolean,
    onResourceClicked?: (gvk: Gvk) => void,
    onPinButtonClicked?: (gvk: Gvk) => void,
    onGvkRightClicked?: (gvk: Gvk) => void,
}

const GvkList: React.FC<GvkListProps> = (props) => {
    const {
        className,
        gvks,
        withGroupName = false,
        onResourceClicked = () => undefined,
        onPinButtonClicked = () => undefined,
        onGvkRightClicked = () => undefined,
    } = props;

    return (
        <div className={[classes.container, className].filter(c => !!c).join(' ')}>
            <ul>
                {
                    gvks.map((gvk) => {
                        const { group, version, kind } = gvk;

                        return (
                            <li key={`${kind}.${group}/${version}`}>
                                <span onClick={() => onResourceClicked(gvk)} onContextMenu={e => {
                                    e.preventDefault();
                                    onGvkRightClicked(gvk);
                                }}>
                                    {
                                        group !== '' && withGroupName
                                            ? `${kind} (${group})`
                                            : `${kind}`
                                    }
                                </span>
                                <button onClick={() => onPinButtonClicked(gvk)}>📌</button>
                            </li>
                        );
                    })
                }
            </ul>
        </div>
    );
};

export default GvkList;
