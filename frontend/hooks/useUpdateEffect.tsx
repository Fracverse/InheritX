import { useEffect, useRef } from "react";

function useUpdateEffect(callback: () => void, dependencies: unknown[]) {
  const isFirstRender = useRef(true);

  useEffect(() => {
    if (isFirstRender.current) {
      isFirstRender.current = false;
      return;
    }
    return callback();
  }, dependencies);
}

export default useUpdateEffect;
