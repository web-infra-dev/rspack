await new Promise(resolve => setTimeout(resolve, 10));
throw new Error("last entry rejected");

export {};
