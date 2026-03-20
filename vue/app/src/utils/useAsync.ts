import { ref, computed, reactive } from 'vue';

// 共享状态接口
export interface PageState {
    isLoading: boolean;
    error: Error | null;
    startRequest: () => void;
    endRequest: () => void;
    setError: (err: Error | null) => void;
    clearError: () => void;
    useAsync: <T>(call: () => Promise<T | undefined>) => Promise<T | undefined>;
}

// 创建页面级共享状态
export default function createPageState(): PageState {
    const loadingCount = ref(0);
    const error = ref<Error | null>(null);

    const isLoading = computed(() => loadingCount.value > 0);

    const startRequest = () => loadingCount.value++;
    const endRequest = () => {
        loadingCount.value--
        loadingCount.value = Math.max(0, loadingCount.value);
    };
    const setError = (err: Error | null) => (error.value = err);
    const clearError = () => (error.value = null);

    const useAsync = async <T>(call: () => Promise<T | undefined>): Promise<T | undefined> => {
        try {
            clearError();        // 使用内部方法
            startRequest();
            return await call();
        } catch (err) {
            setError(err as Error);
        } finally {
            endRequest();
        }
        return undefined;
    };

    return reactive({
        isLoading,
        error,
        startRequest,
        endRequest,
        setError,
        clearError,
        useAsync,
    });
}