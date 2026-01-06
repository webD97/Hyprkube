import React, { PropsWithChildren, useCallback, useEffect, useReducer, useState } from "react";
import { createPortal } from "react-dom";
import { ErrorBoundary } from "react-error-boundary";
import { MegaTabContext } from "../../contexts/MegaTab";
import { MegaTabDefinition, MegaTabsContextType } from "../../contexts/MegaTabs";
import { TabDefinition, TabIdentifier } from "../../hooks/useHeadlessTabs";
import classes from './styles.module.css';

export interface MegaTabsProps {
    onCloseClicked?: (idx: TabIdentifier) => void,
    context: MegaTabsContextType,
    outlet: React.RefObject<HTMLDivElement | null>
}

function MegaTabs(props: PropsWithChildren<MegaTabsProps>) {
    const {
        outlet,
        onCloseClicked = () => undefined,
        context: {
            tabState,
            activeTab,
            switchTab,
            updateTabMeta
        }
    } = props;

    const [closingTabs, setClosingTabs] = useState<TabIdentifier[]>([]);

    const handleClose = useCallback((tabId: TabIdentifier) => {
        setClosingTabs(closingTabs => [...closingTabs, tabId]);
        setTimeout(() => {
            setClosingTabs(closingTabs => closingTabs.filter(current => current !== tabId));
            onCloseClicked(tabId);
        }, 150);
    }, [onCloseClicked]);

    // HACK: re-render once after the ref is populated
    const [, incrementRefreshKey] = useReducer((x: number) => x + 1, 0);
    useEffect(incrementRefreshKey, [incrementRefreshKey]);

    return (
        <div>
            <div className={classes.tabWrapper}>
                {
                    Object.entries(tabState)
                        .map(([idx, tab]) => [idx, tab] as [TabIdentifier, TabDefinition<MegaTabDefinition>])
                        .map(([idx, { meta: { title, icon, immortal, subtitle } }]) => (
                            <div key={idx} className={`${closingTabs.includes(idx) ? classes.fadeOut : ''}`}>
                                <div
                                    title={`${title}${subtitle && ` - ${subtitle}`}`}
                                    className={`${idx === activeTab ? classes.activeTab : ''} ${classes.tab}`}
                                    onClick={() => switchTab(idx)}
                                    onAuxClick={(e) => e.button === 1 && !immortal && handleClose(idx)}
                                >
                                    <span className={classes.tabIcon}>{icon}</span>
                                    <span className={classes.tabLabelWrapper}>
                                        <span className={classes.tabLabel}>{title}</span>
                                        {
                                            subtitle
                                                ? <span className={classes.tabSubtitle}>{subtitle}</span>
                                                : null
                                        }
                                    </span>
                                    {
                                        immortal || Object.entries(tabState).length < 2
                                            ? null
                                            : <span className={classes.closeIcon} title="Close tab" onClick={(e) => {
                                                e.stopPropagation();

                                                if (!immortal) {
                                                    handleClose(idx);
                                                }
                                            }}>🗙</span>
                                    }
                                </div>
                            </div>
                        ))
                }
                {
                    props.children
                }
            </div>
            {
                outlet.current && createPortal(
                    Object.entries(tabState)
                        .map(([idx, tab]) => [idx, tab] as [TabIdentifier, TabDefinition<MegaTabDefinition>])
                        .map(([idx, { meta: { keepAlive }, render }]) => {
                            if (!keepAlive && activeTab !== idx) return;
                            return (
                                <div key={idx} style={{ display: activeTab === idx ? 'initial' : 'none' }}>
                                    <ErrorBoundary
                                        fallbackRender={(context) => (
                                            <div role="alert">
                                                <p>Something went wrong:</p>
                                                <pre style={{ color: "red" }}>{JSON.stringify(context.error, undefined, 2)}</pre>
                                            </div>
                                        )}
                                    >
                                        <MegaTabContext.Provider
                                            value={{
                                                tabIdentifier: idx,
                                                setMeta: updateTabMeta,
                                            }}
                                        >
                                            {render()}
                                        </MegaTabContext.Provider>
                                    </ErrorBoundary>
                                </div>
                            );
                        })
                    , outlet.current
                )
            }
        </div>
    );
};

export default MegaTabs;

export interface MegaTabsButtonProps {
    icon: string,
    title?: string,
    onClick?: React.MouseEventHandler
}

export const MegaTabsButton: React.FC<MegaTabsButtonProps> = (props) => {
    const { icon, title, onClick = () => undefined } = props;

    return (
        <div className={`${classes.tab} ${classes.mini}`} title={title} onClick={onClick}>
            <div className={classes.tabLabel}>{icon}</div>
        </div>
    );
}
