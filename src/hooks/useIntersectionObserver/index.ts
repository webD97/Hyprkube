import { RefObject, useEffect, useRef, useState } from "react";

/**
 * Returns true, if the element behind the ref argument is visible within the viewport.
 * 
 * @param ref 
 * @returns `true` if visible, `false` otherwise
 */
export default function useIntersectionObserver<E extends Element>(ref: RefObject<E | null>) {
    const [visible, setVisible] = useState(false);

    const observerOptions: IntersectionObserverInit = {
        root: null,
        rootMargin: '0px',
        threshold: 0.1
    };

    const observerRef = useRef(
        new IntersectionObserver((entries) => {
            entries.forEach(({ isIntersecting }) => {
                setVisible(isIntersecting);
            });
        }, observerOptions)
    );

    useEffect(() => {
        if (!ref.current) return;
        const element = ref.current;
        const observer = observerRef.current;

        observer.observe(element);

        return () => observer.unobserve(element);
    }, [ref]);

    return visible;
}