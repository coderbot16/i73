pub fn array_to_tuple_mut_16<T>(array: &mut [T; 16]) 
	-> (&mut T, &mut T, &mut T, &mut T, &mut T, &mut T, &mut T, &mut T, &mut T, &mut T, &mut T, &mut T, &mut T, &mut T, &mut T, &mut T) 
	{
		
	let l1 = array.split_at_mut(8);
	let l2 = (
		l1.0.split_at_mut(4),
		l1.1.split_at_mut(4)
	);
	let l2 = (
		(l2.0).0, (l2.0).1,
		(l2.1).0, (l2.1).1
	);
	
	let l3 = (
		l2.0.split_at_mut(2),
		l2.1.split_at_mut(2),
		l2.2.split_at_mut(2),
		l2.3.split_at_mut(2),
	);
	let l3 = (
		(l3.0).0, (l3.0).1,
		(l3.1).0, (l3.1).1,
		(l3.2).0, (l3.2).1,
		(l3.3).0, (l3.3).1
	);
	
	let l4 = (
		l3.0.split_at_mut(1), l3.1.split_at_mut(1),
		l3.2.split_at_mut(1), l3.3.split_at_mut(1),
		l3.4.split_at_mut(1), l3.5.split_at_mut(1),
		l3.6.split_at_mut(1), l3.7.split_at_mut(1),
	);
	
	(
		&mut (l4.0).0[0], &mut (l4.0).1[0], &mut (l4.1).0[0], &mut (l4.1).1[0],
		&mut (l4.2).0[0], &mut (l4.2).1[0], &mut (l4.3).0[0], &mut (l4.3).1[0],
		&mut (l4.4).0[0], &mut (l4.4).1[0], &mut (l4.5).0[0], &mut (l4.5).1[0],
		&mut (l4.6).0[0], &mut (l4.6).1[0], &mut (l4.7).0[0], &mut (l4.7).1[0]
	)
}
	
pub fn array_to_tuple_16<T>(array: &[T; 16]) 
	-> (&T, &T, &T, &T, &T, &T, &T, &T, &T, &T, &T, &T, &T, &T, &T, &T) 
	{
		
	let l1 = array.split_at(8);
	let l2 = (
		l1.0.split_at(4),
		l1.1.split_at(4)
	);
	let l2 = (
		(l2.0).0, (l2.0).1,
		(l2.1).0, (l2.1).1
	);
	
	let l3 = (
		l2.0.split_at(2),
		l2.1.split_at(2),
		l2.2.split_at(2),
		l2.3.split_at(2),
	);
	let l3 = (
		(l3.0).0, (l3.0).1,
		(l3.1).0, (l3.1).1,
		(l3.2).0, (l3.2).1,
		(l3.3).0, (l3.3).1
	);
	
	let l4 = (
		l3.0.split_at(1), l3.1.split_at(1),
		l3.2.split_at(1), l3.3.split_at(1),
		l3.4.split_at(1), l3.5.split_at(1),
		l3.6.split_at(1), l3.7.split_at(1),
	);
	
	(
		&(l4.0).0[0], &(l4.0).1[0], &(l4.1).0[0], &(l4.1).1[0],
		&(l4.2).0[0], &(l4.2).1[0], &(l4.3).0[0], &(l4.3).1[0],
		&(l4.4).0[0], &(l4.4).1[0], &(l4.5).0[0], &(l4.5).1[0],
		&(l4.6).0[0], &(l4.6).1[0], &(l4.7).0[0], &(l4.7).1[0]
	)
}
	
pub fn array_to_tuple_mut_9<T>(array: &mut [T; 9]) 
	-> (&mut T, &mut T, &mut T, &mut T, &mut T, &mut T, &mut T, &mut T, &mut T) 
	{
		
	let l1 = array.split_at_mut(4);
	let l2 = (
		l1.0.split_at_mut(2),
		l1.1.split_at_mut(2)
	);
	let l2 = (
		(l2.0).0, (l2.0).1,
		(l2.1).0, (l2.1).1
	);
	
	let l3 = (
		l2.0.split_at_mut(1),
		l2.1.split_at_mut(1),
		l2.2.split_at_mut(1),
		l2.3.split_at_mut(1),
	);
	let l3 = (
		(l3.0).0, (l3.0).1,
		(l3.1).0, (l3.1).1,
		(l3.2).0, (l3.2).1,
		(l3.3).0, (l3.3).1
	);
	
	let l4 = l3.7.split_at_mut(1);
	
	(
		&mut l3.0[0], &mut l3.1[0], &mut l3.2[0], 
		&mut l3.3[0], &mut l3.4[0], &mut l3.5[0], 
		&mut l3.6[0], &mut l4.0[0], &mut l4.1[0]
	)
}
	
pub fn array_to_tuple_9<T>(array: &[T; 9]) 
	-> (&T, &T, &T, &T, &T, &T, &T, &T, &T) 
	{
		
	let l1 = array.split_at(4);
	let l2 = (
		l1.0.split_at(2),
		l1.1.split_at(2)
	);
	let l2 = (
		(l2.0).0, (l2.0).1,
		(l2.1).0, (l2.1).1
	);
	
	let l3 = (
		l2.0.split_at(1),
		l2.1.split_at(1),
		l2.2.split_at(1),
		l2.3.split_at(1),
	);
	let l3 = (
		(l3.0).0, (l3.0).1,
		(l3.1).0, (l3.1).1,
		(l3.2).0, (l3.2).1,
		(l3.3).0, (l3.3).1
	);
	
	let l4 = l3.7.split_at(1);
	
	(
		&l3.0[0], &l3.1[0], &l3.2[0], 
		&l3.3[0], &l3.4[0], &l3.5[0], 
		&l3.6[0], &l4.0[0], &l4.1[0]
	)
}