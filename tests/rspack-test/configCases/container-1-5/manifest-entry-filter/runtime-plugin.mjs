export default ()=>{
	return {
		name: 'runtime-plugin',
		errorLoadRemote(args) {
      return  () => args.id ;
    },
	}
}
