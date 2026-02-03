import {watch} from "vue";

export interface Claim {
    id: string;
    role: string;
    type: string;
    exp: number;
}

export function useClaim() {
    const claim = useState<Claim|null>("claim");
    return {
        claim,
        subscribe: (callback: (value: any | null) => void) => watch(claim, callback, {immediate: true}),
    };
}